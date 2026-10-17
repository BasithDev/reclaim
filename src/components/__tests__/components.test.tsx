// Tests for key shared components: Dialog/ConfirmDialog, Checkbox, SegmentedControl, AnimatedNumber, Toast, Switch, Skeleton states.
import { describe, expect, it, vi } from "vitest";
import { act, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import {
  AnimatedNumber, Button, Checkbox, ConfirmDialog, Dialog, EmptyState, ProgressBar, SegmentedControl, Switch, ToastProvider, useToast, categoryMeta,
} from "..";
import { Inbox } from "lucide-react";
import { formatBytes } from "../../lib/format";

describe("Dialog", () => {
  it("renders nothing when closed and content when open", () => {
    const { rerender } = render(<Dialog open={false} onClose={() => {}} title="Hello">body</Dialog>);
    expect(screen.queryByRole("dialog")).toBeNull();
    rerender(<Dialog open onClose={() => {}} title="Hello">body text</Dialog>);
    expect(screen.getByRole("dialog", { name: "Hello" })).toBeInTheDocument();
    expect(screen.getByText("body text")).toBeInTheDocument();
  });
  it("closes on Escape and on the close button", async () => {
    const onClose = vi.fn();
    render(<Dialog open onClose={onClose} title="T">x</Dialog>);
    await userEvent.keyboard("{Escape}");
    expect(onClose).toHaveBeenCalledTimes(1);
    await userEvent.click(screen.getByLabelText("Close"));
    expect(onClose).toHaveBeenCalledTimes(2);
  });
});

describe("ConfirmDialog", () => {
  it("calls onConfirm / onClose from its buttons", async () => {
    const onConfirm = vi.fn(), onClose = vi.fn();
    render(<ConfirmDialog open onClose={onClose} onConfirm={onConfirm} title="Delete?" message="Sure?" confirmLabel="Delete" tone="danger" />);
    await userEvent.click(screen.getByRole("button", { name: "Delete" }));
    expect(onConfirm).toHaveBeenCalled();
    await userEvent.click(screen.getByRole("button", { name: "Cancel" }));
    expect(onClose).toHaveBeenCalled();
  });
  it("disables confirm while loading", () => {
    render(<ConfirmDialog open onClose={() => {}} onConfirm={() => {}} title="T" message="m" confirmLabel="Go" loading />);
    expect(screen.getByRole("button", { name: "Go" })).toBeDisabled();
  });
});

describe("Checkbox / Switch", () => {
  it("toggles and reports the new value", async () => {
    const onChange = vi.fn();
    render(<Checkbox checked={false} onChange={onChange} label="Pick me" />);
    await userEvent.click(screen.getByRole("checkbox"));
    expect(onChange).toHaveBeenCalledWith(true);
  });
  it("supports the mixed state", () => {
    render(<Checkbox checked={false} indeterminate onChange={() => {}} />);
    expect(screen.getByRole("checkbox")).toHaveAttribute("aria-checked", "mixed");
  });
  it("Switch reflects and flips state", async () => {
    const onChange = vi.fn();
    render(<Switch checked label="Dark" description="desc" onChange={onChange} />);
    const sw = screen.getByRole("switch");
    expect(sw).toHaveAttribute("aria-checked", "true");
    await userEvent.click(sw);
    expect(onChange).toHaveBeenCalledWith(false);
  });
});

describe("SegmentedControl", () => {
  it("marks the active option and fires onChange", async () => {
    const onChange = vi.fn();
    render(<SegmentedControl value="a" onChange={onChange} options={[{ value: "a", label: "A" }, { value: "b", label: "B" }]} />);
    expect(screen.getByRole("radio", { name: "A" })).toHaveAttribute("aria-checked", "true");
    await userEvent.click(screen.getByRole("radio", { name: "B" }));
    expect(onChange).toHaveBeenCalledWith("b");
  });
});

describe("AnimatedNumber", () => {
  it("renders the formatted initial value and settles on the new one", async () => {
    const { rerender } = render(<AnimatedNumber value={1024} format={formatBytes} duration={0.05} />);
    expect(screen.getByText("1.0 KB")).toBeInTheDocument();
    rerender(<AnimatedNumber value={2048} format={formatBytes} duration={0.05} />);
    await vi.waitFor(() => expect(screen.getByText("2.0 KB")).toBeInTheDocument(), { timeout: 2000 });
  });
});

describe("Toast", () => {
  function Trigger() {
    const { toast } = useToast();
    return <button onClick={() => toast({ title: "Saved", description: "All good", tone: "success" })}>fire</button>;
  }
  it("shows a toast and auto-dismisses it", async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    render(<ToastProvider><Trigger /></ToastProvider>);
    await act(async () => { screen.getByText("fire").click(); });
    expect(screen.getByText("Saved")).toBeInTheDocument();
    expect(screen.getByText("All good")).toBeInTheDocument();
    await act(async () => { vi.advanceTimersByTime(4500); });
    await vi.waitFor(() => expect(screen.queryByText("Saved")).toBeNull());
    vi.useRealTimers();
  });
});

describe("misc", () => {
  it("Button loading disables the button", () => {
    render(<Button loading>Save</Button>);
    expect(screen.getByRole("button")).toBeDisabled();
  });
  it("EmptyState shows title, description and action", () => {
    render(<EmptyState icon={Inbox} title="Nothing" description="Come back later" action={<button>Act</button>} />);
    expect(screen.getByText("Nothing")).toBeInTheDocument();
    expect(screen.getByText("Come back later")).toBeInTheDocument();
    expect(screen.getByText("Act")).toBeInTheDocument();
  });
  it("ProgressBar exposes a clamped value", () => {
    render(<ProgressBar value={1.7} />);
    expect(screen.getByRole("progressbar")).toHaveAttribute("aria-valuenow", "100");
  });
  it("categoryMeta covers every category with a label", () => {
    for (const c of ["video", "audio", "image", "document", "archive", "diskImage", "code", "executable", "database", "log", "other"] as const) {
      expect(categoryMeta[c].label).toBeTruthy();
      expect(typeof categoryMeta[c].color).toBe("string");
    }
  });
});
