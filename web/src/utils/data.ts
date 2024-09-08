import { MacroUsage } from "../../bindings/MacroUsage";

export function usageToCount(usage: MacroUsage) {
  return Object.values(usage).reduce((a, b) => a + b);
}
