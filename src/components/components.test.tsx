import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import ConfirmDialog from "./ConfirmDialog";
import ErrorBanner from "./ErrorBanner";
import PasswordInput from "./PasswordInput";

describe("PasswordInput", () => {
  it("hides the value until shown", () => {
    render(
      <PasswordInput
        id="pw"
        label="Password"
        value="secret value"
        onChange={vi.fn()}
        autoComplete="off"
      />,
    );
    const input = screen.getByLabelText("Password");
    expect(input).toHaveAttribute("type", "password");
    fireEvent.click(screen.getByRole("button", { name: "Show password" }));
    expect(input).toHaveAttribute("type", "text");
    fireEvent.click(screen.getByRole("button", { name: "Hide password" }));
    expect(input).toHaveAttribute("type", "password");
  });

  it("reports changes", () => {
    const onChange = vi.fn();
    render(
      <PasswordInput
        id="pw"
        label="Password"
        value=""
        onChange={onChange}
        autoComplete="off"
      />,
    );
    fireEvent.change(screen.getByLabelText("Password"), {
      target: { value: "x" },
    });
    expect(onChange).toHaveBeenCalledWith("x");
  });
});

describe("ErrorBanner", () => {
  it("renders nothing without a message", () => {
    const { container } = render(<ErrorBanner message={null} />);
    expect(container).toBeEmptyDOMElement();
  });

  it("announces the message and can be dismissed", () => {
    const onDismiss = vi.fn();
    render(<ErrorBanner message="Disk full." onDismiss={onDismiss} />);
    expect(screen.getByRole("alert")).toHaveTextContent("Disk full.");
    fireEvent.click(screen.getByRole("button", { name: "Dismiss error" }));
    expect(onDismiss).toHaveBeenCalledOnce();
  });
});

describe("ConfirmDialog", () => {
  function renderDialog(busy = false) {
    const onConfirm = vi.fn();
    const onCancel = vi.fn();
    render(
      <ConfirmDialog
        title="Delete entry?"
        message="It will be deleted."
        confirmLabel="Delete"
        onConfirm={onConfirm}
        onCancel={onCancel}
        busy={busy}
      />,
    );
    return { onConfirm, onCancel };
  }

  it("focuses Cancel and confirms on click", () => {
    const { onConfirm } = renderDialog();
    expect(screen.getByRole("button", { name: "Cancel" })).toHaveFocus();
    fireEvent.click(screen.getByRole("button", { name: "Delete" }));
    expect(onConfirm).toHaveBeenCalledOnce();
  });

  it("cancels with Escape", () => {
    const { onCancel } = renderDialog();
    fireEvent.keyDown(window, { key: "Escape" });
    expect(onCancel).toHaveBeenCalledOnce();
  });

  it("ignores Escape and disables buttons while busy", () => {
    const { onCancel } = renderDialog(true);
    fireEvent.keyDown(window, { key: "Escape" });
    expect(onCancel).not.toHaveBeenCalled();
    expect(screen.getByRole("button", { name: "Delete" })).toBeDisabled();
  });
});
