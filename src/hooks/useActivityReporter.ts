/** Hook that tells the core when the user is active, to postpone the auto-lock. */
import { useEffect } from "react";
import { touchActivity } from "../api/security";

/** Minimum time between two activity reports, in milliseconds. */
export const ACTIVITY_REPORT_INTERVAL_MS = 15_000;

/** Window events that count as user activity. */
const ACTIVITY_EVENTS = [
  "keydown",
  "mousedown",
  "mousemove",
  "wheel",
  "touchstart",
] as const;

/**
 * Reports keyboard, mouse and touch activity to the core, at most once per
 * {@link ACTIVITY_REPORT_INTERVAL_MS}, so the automatic lock only happens
 * after real inactivity.
 */
export function useActivityReporter() {
  useEffect(() => {
    let lastReport = 0;

    function report() {
      const now = Date.now();
      if (now - lastReport < ACTIVITY_REPORT_INTERVAL_MS) {
        return;
      }
      lastReport = now;
      // A lost report can only make the lock happen sooner, never later,
      // so failures are safe to ignore.
      touchActivity().catch(() => undefined);
    }

    for (const name of ACTIVITY_EVENTS) {
      window.addEventListener(name, report, { passive: true });
    }
    return () => {
      for (const name of ACTIVITY_EVENTS) {
        window.removeEventListener(name, report);
      }
    };
  }, []);
}
