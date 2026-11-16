/**
 * End-to-end flows of the interface: the whole app against a stateful fake
 * of the core, driven only through what the user sees.
 */
import { act, fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import App from "../App";
import { VAULT_LOCKED_EVENT } from "../api/security";
import type { Entry } from "../api/types";
import { emitEvent } from "../test/tauriMock";
import { installFakeCore } from "../test/fakeCore";

const MASTER = "synthetic master phrase";
const NEW_MASTER = "another synthetic phrase";

function type(label: string, value: string) {
  fireEvent.change(screen.getByLabelText(label), { target: { value } });
}

function click(name: string) {
  fireEvent.click(screen.getByRole("button", { name }));
}

async function unlock(password: string) {
  type("Master password", password);
  click("Unlock");
  await screen.findByRole("tab", { name: "Passwords" });
}

async function lock() {
  click("Lock");
  await screen.findByText("Your vault is locked.");
}

const SAVED: Entry = {
  id: "f".repeat(32),
  title: "Saved mail",
  username: "saved@example.test",
  password: "saved-password",
  url: "",
  notes: "",
  createdAt: 0,
  updatedAt: 0,
};

describe("first run", () => {
  it("creates a vault, stores an entry, copies it and finds it after locking", async () => {
    const core = installFakeCore();
    render(<App />);

    await screen.findByRole("heading", { name: "Create your vault" });
    type("Master password", MASTER);
    type("Confirm master password", MASTER);
    fireEvent.click(screen.getByRole("checkbox"));
    click("Create vault");

    await screen.findByRole("button", { name: "New entry" });
    click("New entry");
    type("Title", "Example mail");
    type("Username", "user@example.test");
    type("Password", "synthetic-entry-password");
    click("Add entry");
    await screen.findByRole("heading", { name: "Example mail" });

    click("Copy password");
    expect(await screen.findByRole("status")).toHaveTextContent(
      "Password copied.",
    );
    expect(core.clipboard).toBe("synthetic-entry-password");

    await lock();
    expect(core.clipboard).toBeNull();
    expect(screen.queryByText("Example mail")).not.toBeInTheDocument();

    await unlock(MASTER);
    expect(
      await screen.findByRole("button", { name: /Example mail/ }),
    ).toBeVisible();
  });
});

describe("returning user", () => {
  it("refuses a wrong master password and then unlocks", async () => {
    installFakeCore({
      exists: true,
      password: MASTER,
      entries: new Map([[SAVED.id, SAVED]]),
    });
    render(<App />);

    await screen.findByText("Your vault is locked.");
    type("Master password", "not the synthetic phrase");
    click("Unlock");
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Incorrect master password.",
    );

    await unlock(MASTER);
    expect(
      await screen.findByRole("button", { name: /Saved mail/ }),
    ).toBeVisible();
  });

  it("changes the master password and needs the new one afterwards", async () => {
    const core = installFakeCore({
      exists: true,
      unlocked: true,
      password: MASTER,
    });
    render(<App />);

    fireEvent.click(await screen.findByRole("tab", { name: "Settings" }));
    type("Current master password", MASTER);
    type("New master password", NEW_MASTER);
    type("Confirm new master password", NEW_MASTER);
    click("Change password");
    expect(await screen.findByRole("status")).toHaveTextContent(
      "Master password changed.",
    );
    expect(core.password).toBe(NEW_MASTER);

    await lock();
    type("Master password", MASTER);
    click("Unlock");
    await screen.findByRole("alert");
    await unlock(NEW_MASTER);
  });

  it("explains a lock after inactivity and forgets what was shown", async () => {
    const core = installFakeCore({
      exists: true,
      unlocked: true,
      password: MASTER,
      entries: new Map([[SAVED.id, SAVED]]),
    });
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: /Saved mail/ }));
    await screen.findByRole("heading", { name: "Saved mail" });

    core.unlocked = false;
    act(() => emitEvent(VAULT_LOCKED_EVENT, { reason: "idle" }));

    expect(await screen.findByRole("status")).toHaveTextContent(
      "The vault was locked after 5 minutes of inactivity.",
    );
    expect(screen.queryByText("Saved mail")).not.toBeInTheDocument();
    expect(screen.queryByText("saved@example.test")).not.toBeInTheDocument();
  });
});

describe("new computer", () => {
  it("restores a backup instead of creating a vault", async () => {
    installFakeCore({
      backup: { password: MASTER, entries: [SAVED] },
    });
    render(<App />);

    await screen.findByRole("heading", { name: "Create your vault" });
    click("Restore from a backup instead");
    type("Backup master password", MASTER);
    click("Choose backup and restore");
    expect(await screen.findByRole("status")).toHaveTextContent(
      "Backup restored.",
    );

    await unlock(MASTER);
    expect(
      await screen.findByRole("button", { name: /Saved mail/ }),
    ).toBeVisible();
  });
});
