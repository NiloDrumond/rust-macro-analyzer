import { MacroDefinitionCountByType } from "./charts/macro-definition-count-by-type";
import { MacroDefinitionsPerCrate } from "./charts/macro-definitions-per-crate";
import { MacroDefinitionsPerRepo } from "./charts/macro-definitions-per-repo";
import { MacroInvocationCountByType } from "./charts/macro-invocation-count-by-type";
import { MacroInvocationsPerCrate } from "./charts/macro-invocations-per-crate";
import { MacroInvocationsPerRepo } from "./charts/macro-invocations-per-repo";
import { MostUsedBuiltinAttributeMacro } from "./charts/most-used-builtin-attribute-macros";

export function Results() {
  return (
    <div className="grid grid-cols-2 w-full overflow-y-auto p-6 gap-4">
      <div className="flex flex-col col-span-2">
        <h1 className="mb-2">Rust Macro Analyzis</h1>
        <p>
          All data extracted from the 100 github rust repositories with most
          stargazers
        </p>
        <p>As of: 23/08/2024</p>
      </div>
      <MacroInvocationCountByType />
      <MacroDefinitionCountByType />
      <MostUsedBuiltinAttributeMacro />
      <MacroInvocationsPerRepo />
      <MacroDefinitionsPerRepo />
      <MacroInvocationsPerCrate />
      <MacroDefinitionsPerCrate />
    </div>
  );
}
