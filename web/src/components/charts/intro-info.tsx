import React from "react";
import { useData } from "../../hooks/use-data";
import { usageToCount } from "../../utils/data";
import { Card } from "../ui/card";
import { format } from "date-fns";

export function IntroInfo() {
  const { data } = useData();

  const { crates, definitions, invocations } = React.useMemo(() => {
    if (!data) return { crates: 0, invocations: 0, definitions: 0 };
    const {
      derive_macro_definitions,
      procedural_macro_definitions,
      attribute_macro_definitions,
      declarative_macro_definitions,
      macro_invocations,
      builtin_attribute_macro_invocations,
      attribute_macro_invocations,
      derive_macro_usage,
    } = data.total_macro_usage;
    const invocations =
      usageToCount(macro_invocations) +
      usageToCount(builtin_attribute_macro_invocations) +
      usageToCount(attribute_macro_invocations) +
      usageToCount(derive_macro_usage.derives);
    const definitions =
      usageToCount(derive_macro_definitions) +
      usageToCount(attribute_macro_definitions) +
      usageToCount(declarative_macro_definitions) +
      usageToCount(procedural_macro_definitions);
    return {
      crates: data.macro_invocations_per_crate.length,
      invocations,
      definitions,
    };
  }, [data]);

  return (
    <Card>
      <h1 className="mb-2">Rust Macro Analyzis</h1>
      <p>
        All data extracted from the 100 github rust repositories with most
        stargazers
      </p>
      <p>As of: {format(new Date(data?.date ?? 0), "P")}</p>
      <p>
        <strong>{crates}</strong> crates analyzed
      </p>
      <p>
        <strong>{invocations}</strong> macro invocations
      </p>
      <p>
        <strong>{definitions}</strong> macro definitions
      </p>
    </Card>
  );
}
