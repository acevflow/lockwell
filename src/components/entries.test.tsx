import { fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ApiError } from "../api/client";
import type { Entry, EntrySummary } from "../api/types";
import { ENTRY_LIMITS } from "../entryRules";
import EntryForm from "./EntryForm";
import EntryList from "./EntryList";
import EntryView from "./EntryView";

/** A fixed timestamp, so the tests never depend on the clock. */
const UPDATED_AT = Date.UTC(2026, 0, 2, 3, 4, 5);

const ENTRY: Entry = {
  id: "0123456789abcdef0123456789abcdef",
  title: "Example mail",
  username: "user@example.test",
  password: "synthetic-password",
  url: "https://mail.example.test",
  notes: "Synthetic notes",
  createdAt: UPDATED_AT - 1000,
  updatedAt: UPDATED_AT,
};

const SUMMARIES: EntrySummary[] = [
  {
    id: "a".repeat(32),
    title: "Bank",
    username: "me",
    url: "",
    updatedAt: UPDATED_AT,
  },
  {
    id: "b".repeat(32),
    title: "Forum",
    username: "",
    url: "",
    updatedAt: UPDATED_AT,
  },
];

function renderForm(props: Partial<Parameters<typeof EntryForm>[0]> = {}) {
  const onSubmit = vi.fn(() => Promise.resolve());
  const onCancel = vi.fn();
  render(
    <EntryForm
      heading="New entry"
      submitLabel="Save"
      onSubmit={onSubmit}
      onCancel={onCancel}
      {...props}
    />,
  );
  return { onSubmit, onCancel };
}

function renderView(entry: Entry = ENTRY) {
  const handlers = {
    onEdit: vi.fn(),
    onDelete: vi.fn(),
    onCopy: vi.fn(() => Promise.resolve()),
  };
  render(<EntryView entry={entry} {...handlers} />);
  return handlers;
}

describe("EntryForm", () => {
  it("submits only the editable fields of the initial entry", async () => {
    const { onSubmit } = renderForm({ initial: ENTRY });

    fireEvent.change(screen.getByLabelText("Title"), {
      target: { value: "Renamed" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Save" }));

    await vi.waitFor(() =>
      expect(onSubmit).toHaveBeenCalledExactlyOnceWith({
        title: "Renamed",
        username: ENTRY.username,
        password: ENTRY.password,
        url: ENTRY.url,
        notes: ENTRY.notes,
      }),
    );
  });

  it("starts empty without an initial entry", () => {
    renderForm();

    expect(screen.getByLabelText("Title")).toHaveValue("");
    expect(screen.getByLabelText("Username")).toHaveValue("");
    expect(screen.getByLabelText("Password")).toHaveValue("");
    expect(screen.getByLabelText("Website")).toHaveValue("");
    expect(screen.getByLabelText("Notes")).toHaveValue("");
  });

  it("limits every field to what the core accepts", () => {
    renderForm();

    expect(screen.getByLabelText("Title")).toHaveAttribute(
      "maxLength",
      String(ENTRY_LIMITS.title),
    );
    expect(screen.getByLabelText("Username")).toHaveAttribute(
      "maxLength",
      String(ENTRY_LIMITS.username),
    );
    expect(screen.getByLabelText("Website")).toHaveAttribute(
      "maxLength",
      String(ENTRY_LIMITS.url),
    );
    expect(screen.getByLabelText("Notes")).toHaveAttribute(
      "maxLength",
      String(ENTRY_LIMITS.notes),
    );
  });

  it("requires a title that is not only spaces", () => {
    const { onSubmit } = renderForm();

    fireEvent.change(screen.getByLabelText("Title"), {
      target: { value: "   " },
    });
    fireEvent.click(screen.getByRole("button", { name: "Save" }));

    expect(screen.getByRole("alert")).toHaveTextContent(
      "The title is required.",
    );
    expect(onSubmit).not.toHaveBeenCalled();
  });

  it("shows a save error and stays usable", async () => {
    const onSubmit = vi.fn(() =>
      Promise.reject(new ApiError("invalid_input", "the title is too long")),
    );
    renderForm({ initial: ENTRY, onSubmit });

    fireEvent.click(screen.getByRole("button", { name: "Save" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "The title is too long.",
    );
    expect(screen.getByRole("button", { name: "Save" })).toBeEnabled();
    expect(screen.getByRole("button", { name: "Cancel" })).toBeEnabled();
  });

  it("disables both buttons while saving", () => {
    renderForm({ initial: ENTRY, onSubmit: () => new Promise(() => {}) });

    fireEvent.click(screen.getByRole("button", { name: "Save" }));

    expect(screen.getByRole("button", { name: "Saving…" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Cancel" })).toBeDisabled();
  });

  it("cancels without saving", () => {
    const { onSubmit, onCancel } = renderForm({ initial: ENTRY });

    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));

    expect(onCancel).toHaveBeenCalledOnce();
    expect(onSubmit).not.toHaveBeenCalled();
  });
});

describe("EntryList", () => {
  it("shows the empty message without a list", () => {
    render(
      <EntryList
        entries={[]}
        selectedId={null}
        onSelect={vi.fn()}
        emptyMessage="No entries yet."
      />,
    );

    expect(screen.getByText("No entries yet.")).toBeInTheDocument();
    expect(screen.queryByRole("list")).not.toBeInTheDocument();
  });

  it("lists titles and usernames and marks the selected entry", () => {
    render(
      <EntryList
        entries={SUMMARIES}
        selectedId={SUMMARIES[1].id}
        onSelect={vi.fn()}
        emptyMessage="No entries yet."
      />,
    );

    const items = within(
      screen.getByRole("list", { name: "Entries" }),
    ).getAllByRole("button");
    expect(items.map((item) => item.textContent)).toEqual(["Bankme", "Forum"]);
    expect(items[0]).not.toHaveAttribute("aria-current");
    expect(items[1]).toHaveAttribute("aria-current", "true");
  });

  it("selects an entry by its id", () => {
    const onSelect = vi.fn();
    render(
      <EntryList
        entries={SUMMARIES}
        selectedId={null}
        onSelect={onSelect}
        emptyMessage="No entries yet."
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: /Bank/ }));

    expect(onSelect).toHaveBeenCalledExactlyOnceWith(SUMMARIES[0].id);
  });
});

describe("EntryView", () => {
  it("shows every field with the password masked", () => {
    renderView();

    expect(
      screen.getByRole("heading", { name: "Example mail" }),
    ).toBeInTheDocument();
    expect(screen.getByText(ENTRY.username)).toBeInTheDocument();
    expect(screen.getByText(ENTRY.url)).toBeInTheDocument();
    expect(screen.getByText(ENTRY.notes)).toBeInTheDocument();
    expect(screen.getByTestId("entry-password")).toHaveTextContent(
      "••••••••••••",
    );
    expect(screen.queryByText(ENTRY.password)).not.toBeInTheDocument();
  });

  it("shows and hides the password again", () => {
    renderView();

    fireEvent.click(screen.getByRole("button", { name: "Show" }));
    expect(screen.getByTestId("entry-password")).toHaveTextContent(
      ENTRY.password,
    );
    expect(screen.getByRole("button", { name: "Hide" })).toHaveAttribute(
      "aria-pressed",
      "true",
    );

    fireEvent.click(screen.getByRole("button", { name: "Hide" }));
    expect(screen.getByTestId("entry-password")).toHaveTextContent(
      "••••••••••••",
    );
  });

  it("shows None for empty fields and offers nothing to copy", () => {
    renderView({ ...ENTRY, username: "", password: "", url: "", notes: "" });

    expect(screen.getAllByText("None")).toHaveLength(4);
    expect(
      screen.queryByRole("button", { name: /Copy/ }),
    ).not.toBeInTheDocument();
    expect(screen.queryByTestId("entry-password")).not.toBeInTheDocument();
  });

  it("says when the username was copied and when it will be cleared", async () => {
    const { onCopy } = renderView();

    fireEvent.click(screen.getByRole("button", { name: "Copy username" }));

    expect(await screen.findByRole("status")).toHaveTextContent(
      "Username copied. It will be cleared from the clipboard in 30 seconds.",
    );
    expect(onCopy).toHaveBeenCalledExactlyOnceWith("username");
  });

  it("says nothing when copying fails", async () => {
    const onCopy = vi.fn(() => Promise.reject(new Error("unavailable")));
    render(
      <EntryView
        entry={ENTRY}
        onEdit={vi.fn()}
        onDelete={vi.fn()}
        onCopy={onCopy}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Copy password" }));

    await vi.waitFor(() => expect(onCopy).toHaveBeenCalledOnce());
    expect(screen.queryByRole("status")).not.toBeInTheDocument();
  });

  it("shows when the entry last changed", () => {
    renderView();

    expect(
      screen.getByText(new Date(UPDATED_AT).toLocaleString()),
    ).toBeInTheDocument();
  });

  it("asks to edit or delete", () => {
    const { onEdit, onDelete } = renderView();

    fireEvent.click(screen.getByRole("button", { name: "Edit" }));
    fireEvent.click(screen.getByRole("button", { name: "Delete" }));

    expect(onEdit).toHaveBeenCalledOnce();
    expect(onDelete).toHaveBeenCalledOnce();
  });
});
