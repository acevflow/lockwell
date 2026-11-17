/** Tests of the Passwords tab against an in-memory stand-in for the core. */
import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Entry, EntryInput } from "../api/types";
import { callsTo, failWith, mockCommands } from "../test/tauriMock";
import PasswordsTab from "./PasswordsTab";

const ID_A = "a".repeat(32);
const ID_B = "b".repeat(32);

/** An in-memory stand-in for the core, enough to drive the tab. */
function fakeCore() {
  const store = new Map<string, Entry>([
    [ID_A, makeEntry(ID_A, "Mail", "me@example.com", "mail-pass")],
    [ID_B, makeEntry(ID_B, "Bank", "customer", "bank-pass")],
  ]);
  let next = 0;
  return mockCommands({
    list_entries: () =>
      [...store.values()]
        .sort((x, y) => x.title.localeCompare(y.title))
        .map(({ id, title, username, url, updatedAt }) => ({
          id,
          title,
          username,
          url,
          updatedAt,
        })),
    get_entry: ({ id }) => {
      const entry = store.get(id as string);
      if (!entry) throw { code: "not_found", message: "entry not found" };
      return entry;
    },
    create_entry: ({ entry }) => {
      rejectUnknownFields(entry);
      const id = String(++next).padStart(32, "c");
      store.set(id, {
        ...(entry as EntryInput),
        id,
        createdAt: 1,
        updatedAt: 1,
      });
      return id;
    },
    update_entry: ({ id, entry }) => {
      rejectUnknownFields(entry);
      const existing = store.get(id as string)!;
      store.set(id as string, { ...existing, ...(entry as EntryInput) });
      return null;
    },
    delete_entry: ({ id }) => {
      store.delete(id as string);
      return null;
    },
  });
}

/** Mirrors `deny_unknown_fields` on `EntryInput` in the core. */
function rejectUnknownFields(entry: unknown) {
  const allowed = ["title", "username", "password", "url", "notes"];
  const unknown = Object.keys(entry as object).filter(
    (key) => !allowed.includes(key),
  );
  if (unknown.length > 0) {
    throw { code: "invalid_input", message: `unknown field ${unknown[0]}` };
  }
}

/** A complete entry with empty URL and notes and fixed timestamps. */
function makeEntry(
  id: string,
  title: string,
  username: string,
  password: string,
): Entry {
  return {
    id,
    title,
    username,
    password,
    url: "",
    notes: "",
    createdAt: 1,
    updatedAt: 1,
  };
}

/** The entries list in the sidebar. */
function entryList() {
  return screen.getByRole("list", { name: "Entries" });
}

/** The fake core, with `command` failing with a core error. */
function coreFailing(command: string, code: string, message: string) {
  const invoke = fakeCore();
  const handlers = invoke.getMockImplementation()!;
  const failure = failWith(code, message);
  invoke.mockImplementation(async (name, args) =>
    name === command ? failure({}) : handlers(name, args),
  );
}

describe("PasswordsTab", () => {
  beforeEach(() => {
    fakeCore();
  });

  it("lists entries without fetching passwords", async () => {
    render(<PasswordsTab onLocked={vi.fn()} />);
    expect(await screen.findByRole("button", { name: /Mail/ })).toBeVisible();
    expect(within(entryList()).getAllByRole("button")).toHaveLength(2);
    expect(callsTo("get_entry")).toHaveLength(0);
    expect(screen.queryByText("mail-pass")).not.toBeInTheDocument();
  });

  it("filters the list with the search box", async () => {
    render(<PasswordsTab onLocked={vi.fn()} />);
    await screen.findByRole("button", { name: /Mail/ });
    fireEvent.change(screen.getByLabelText("Search entries"), {
      target: { value: "bank" },
    });
    expect(within(entryList()).getAllByRole("button")).toHaveLength(1);
    fireEvent.change(screen.getByLabelText("Search entries"), {
      target: { value: "nothing" },
    });
    expect(screen.getByText("No entries match your search.")).toBeVisible();
  });

  it("shows an entry with the password masked until revealed", async () => {
    render(<PasswordsTab onLocked={vi.fn()} />);
    fireEvent.click(await screen.findByRole("button", { name: /Mail/ }));
    expect(await screen.findByRole("heading", { name: "Mail" })).toBeVisible();
    const password = screen.getByTestId("entry-password");
    expect(password).not.toHaveTextContent("mail-pass");
    fireEvent.click(screen.getByRole("button", { name: "Show" }));
    expect(password).toHaveTextContent("mail-pass");
  });

  it("copies the password through the core and says when it is cleared", async () => {
    const invoke = fakeCore();
    const handlers = invoke.getMockImplementation()!;
    invoke.mockImplementation((command, args) =>
      command === "copy_entry_field"
        ? Promise.resolve(null)
        : handlers(command, args),
    );
    render(<PasswordsTab onLocked={vi.fn()} />);
    fireEvent.click(await screen.findByRole("button", { name: /Mail/ }));
    fireEvent.click(
      await screen.findByRole("button", { name: "Copy password" }),
    );
    expect(await screen.findByRole("status")).toHaveTextContent(
      "Password copied. It will be cleared from the clipboard in 30 seconds.",
    );
    expect(callsTo("copy_entry_field")).toEqual([
      { id: ID_A, field: "password" },
    ]);
  });

  it("reports copy failures", async () => {
    const invoke = fakeCore();
    const handlers = invoke.getMockImplementation()!;
    invoke.mockImplementation((command, args) =>
      command === "copy_entry_field"
        ? Promise.reject({
            code: "io",
            message: "the clipboard is not available",
          })
        : handlers(command, args),
    );
    render(<PasswordsTab onLocked={vi.fn()} />);
    fireEvent.click(await screen.findByRole("button", { name: /Mail/ }));
    fireEvent.click(
      await screen.findByRole("button", { name: "Copy username" }),
    );
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "The clipboard is not available.",
    );
    expect(screen.queryByRole("status")).not.toBeInTheDocument();
  });

  it("creates an entry and shows it", async () => {
    render(<PasswordsTab onLocked={vi.fn()} />);
    await screen.findByRole("button", { name: /Mail/ });
    fireEvent.click(screen.getByRole("button", { name: "New entry" }));
    fireEvent.change(screen.getByLabelText("Title"), {
      target: { value: "Forum" },
    });
    fireEvent.change(screen.getByLabelText("Password"), {
      target: { value: "forum-pass" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Add entry" }));
    expect(await screen.findByRole("heading", { name: "Forum" })).toBeVisible();
    expect(callsTo("create_entry")[0].entry).toMatchObject({
      title: "Forum",
      password: "forum-pass",
    });
    expect(within(entryList()).getAllByRole("button")).toHaveLength(3);
  });

  it("requires a title", async () => {
    render(<PasswordsTab onLocked={vi.fn()} />);
    fireEvent.click(await screen.findByRole("button", { name: "New entry" }));
    fireEvent.click(screen.getByRole("button", { name: "Add entry" }));
    expect(screen.getByRole("alert")).toHaveTextContent(
      "The title is required.",
    );
    expect(callsTo("create_entry")).toHaveLength(0);
  });

  it("edits an entry", async () => {
    render(<PasswordsTab onLocked={vi.fn()} />);
    fireEvent.click(await screen.findByRole("button", { name: /Bank/ }));
    fireEvent.click(await screen.findByRole("button", { name: "Edit" }));
    expect(screen.getByLabelText("Title")).toHaveValue("Bank");
    fireEvent.change(screen.getByLabelText("Title"), {
      target: { value: "My bank" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Save changes" }));
    expect(
      await screen.findByRole("heading", { name: "My bank" }),
    ).toBeVisible();
    expect(callsTo("update_entry")[0]).toMatchObject({ id: ID_B });
  });

  it("deletes an entry after confirmation", async () => {
    render(<PasswordsTab onLocked={vi.fn()} />);
    fireEvent.click(await screen.findByRole("button", { name: /Mail/ }));
    fireEvent.click(await screen.findByRole("button", { name: "Delete" }));
    const dialog = screen.getByRole("alertdialog");
    fireEvent.click(within(dialog).getByRole("button", { name: "Delete" }));
    await waitFor(() =>
      expect(within(entryList()).getAllByRole("button")).toHaveLength(1),
    );
    expect(callsTo("delete_entry")).toEqual([{ id: ID_A }]);
    expect(screen.queryByRole("alertdialog")).not.toBeInTheDocument();
  });

  it("keeps the entry when deletion is cancelled", async () => {
    render(<PasswordsTab onLocked={vi.fn()} />);
    fireEvent.click(await screen.findByRole("button", { name: /Mail/ }));
    fireEvent.click(await screen.findByRole("button", { name: "Delete" }));
    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    expect(screen.queryByRole("alertdialog")).not.toBeInTheDocument();
    expect(callsTo("delete_entry")).toHaveLength(0);
  });

  it("shows save errors inside the form", async () => {
    mockCommands({
      list_entries: () => [],
      create_entry: failWith(
        "invalid_input",
        "the notes must have at most 20000 characters",
      ),
    });
    render(<PasswordsTab onLocked={vi.fn()} />);
    expect(await screen.findByText(/No entries yet/)).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "New entry" }));
    fireEvent.change(screen.getByLabelText("Title"), {
      target: { value: "X" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Add entry" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("at most 20000");
  });

  it("calls onLocked when the vault is locked", async () => {
    mockCommands({ list_entries: failWith("locked", "the vault is locked") });
    const onLocked = vi.fn();
    render(<PasswordsTab onLocked={onLocked} />);
    await waitFor(() => expect(onLocked).toHaveBeenCalledOnce());
  });

  it("shows load errors", async () => {
    mockCommands({
      list_entries: failWith("corrupt", "an entry failed to decrypt"),
    });
    render(<PasswordsTab onLocked={vi.fn()} />);
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "An entry failed to decrypt.",
    );
  });

  // The entry is already saved when the refresh fails, so it is still shown.
  it("shows the new entry and an error when the list cannot be refreshed", async () => {
    render(<PasswordsTab onLocked={vi.fn()} />);
    await screen.findByRole("button", { name: /Mail/ });
    coreFailing("list_entries", "storage", "the database is busy");
    fireEvent.click(screen.getByRole("button", { name: "New entry" }));
    fireEvent.change(screen.getByLabelText("Title"), {
      target: { value: "Forum" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Add entry" }));
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "The database is busy.",
    );
    expect(screen.getByRole("heading", { name: "Forum" })).toBeVisible();
  });

  it("shows that the list is loading", () => {
    mockCommands({ list_entries: () => new Promise(() => {}) });
    render(<PasswordsTab onLocked={vi.fn()} />);
    expect(screen.getByText("Loading…")).toBeVisible();
  });

  it("starts without a selected entry", async () => {
    render(<PasswordsTab onLocked={vi.fn()} />);
    await screen.findByRole("button", { name: /Mail/ });
    expect(screen.getByText("Select an entry or add a new one.")).toBeVisible();
  });

  it("marks the selected entry in the list", async () => {
    render(<PasswordsTab onLocked={vi.fn()} />);
    fireEvent.click(await screen.findByRole("button", { name: /Mail/ }));
    await screen.findByRole("heading", { name: "Mail" });
    expect(
      within(entryList()).getByRole("button", { name: /Mail/ }),
    ).toHaveAttribute("aria-current", "true");
  });

  it("shows an error when an entry cannot be opened", async () => {
    coreFailing("get_entry", "corrupt", "an entry failed to decrypt");
    render(<PasswordsTab onLocked={vi.fn()} />);
    fireEvent.click(await screen.findByRole("button", { name: /Mail/ }));
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "An entry failed to decrypt.",
    );
    expect(screen.queryByRole("heading", { name: "Mail" })).toBeNull();
  });

  it("keeps the entry and explains when deletion fails", async () => {
    coreFailing("delete_entry", "storage", "the database is busy");
    render(<PasswordsTab onLocked={vi.fn()} />);
    fireEvent.click(await screen.findByRole("button", { name: /Mail/ }));
    fireEvent.click(await screen.findByRole("button", { name: "Delete" }));
    const dialog = screen.getByRole("alertdialog");
    fireEvent.click(within(dialog).getByRole("button", { name: "Delete" }));
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "The database is busy.",
    );
    expect(screen.queryByRole("alertdialog")).not.toBeInTheDocument();
    expect(within(entryList()).getAllByRole("button")).toHaveLength(2);
  });

  it("dismisses an error", async () => {
    coreFailing("get_entry", "corrupt", "an entry failed to decrypt");
    render(<PasswordsTab onLocked={vi.fn()} />);
    fireEvent.click(await screen.findByRole("button", { name: /Mail/ }));
    await screen.findByRole("alert");
    fireEvent.click(screen.getByRole("button", { name: "Dismiss error" }));
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it("returns to the entry when editing is cancelled", async () => {
    render(<PasswordsTab onLocked={vi.fn()} />);
    fireEvent.click(await screen.findByRole("button", { name: /Mail/ }));
    fireEvent.click(await screen.findByRole("button", { name: "Edit" }));
    fireEvent.change(screen.getByLabelText("Title"), {
      target: { value: "Discarded" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    expect(screen.getByRole("heading", { name: "Mail" })).toBeVisible();
    expect(callsTo("update_entry")).toHaveLength(0);
  });

  it("closes the new entry form when cancelled", async () => {
    render(<PasswordsTab onLocked={vi.fn()} />);
    await screen.findByRole("button", { name: /Mail/ });
    fireEvent.click(screen.getByRole("button", { name: "New entry" }));
    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    expect(screen.getByText("Select an entry or add a new one.")).toBeVisible();
    expect(callsTo("create_entry")).toHaveLength(0);
  });
});
