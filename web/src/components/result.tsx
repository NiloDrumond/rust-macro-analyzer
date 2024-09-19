import { DeriveMacroUsage } from "./charts/derive-macro-usage";
import { IntroInfo } from "./charts/intro-info";
import { LinesPerRepo } from "./charts/lines-per-repository";
import { MacroDefinitionCountByType } from "./charts/macro-definition-count-by-type";
import { MacroDefinitionsPerCrate } from "./charts/macro-definitions-per-crate";
import { MacroDefinitionsPerCrateLinesNormalized } from "./charts/macro-definitions-per-crate-lines-normalized";
import { MacroDefinitionsPerRepo } from "./charts/macro-definitions-per-repo";
import { MacroDefinitionsPerRepoLinesNormalized } from "./charts/macro-definitions-per-repo-lines-normalized";
import { MacroInvocationCountByType } from "./charts/macro-invocation-count-by-type";
import { MacroInvocationsPerCrate } from "./charts/macro-invocations-per-crate";
import { MacroInvocationsPerCrateLinesNormalized } from "./charts/macro-invocations-per-crate-lines-normalized";
import { MacroInvocationsPerRepo } from "./charts/macro-invocations-per-repo";
import { MacroInvocationsPerRepoLinesNormalized } from "./charts/macro-invocations-per-repo-lines-normalized";
import { MostUsedBuiltinAttributeMacros } from "./charts/most-used-builtin-attribute-macros";
import { MostUsedCustomAttributeMacros } from "./charts/most-used-custom-attribute-macros";
import { MostUsedDeriveMacros } from "./charts/most-used-derive-macros";
import { MostUsedFunctionLikeAndDeclarativeMacros } from "./charts/most-used-function-declarative-macros";

export function Results() {
  return (
    <div className="grid grid-cols-2 w-full overflow-y-auto p-6 gap-4">
      <IntroInfo />
      {/* <MiscInfo /> */}
      <DeriveMacroUsage />
      <MacroInvocationCountByType />
      <MacroDefinitionCountByType />
      <MostUsedBuiltinAttributeMacros />
      <MostUsedCustomAttributeMacros />
      <MostUsedFunctionLikeAndDeclarativeMacros />
      <MostUsedDeriveMacros />
      <MacroInvocationsPerRepo />
      <MacroDefinitionsPerRepo />
      <MacroInvocationsPerRepoLinesNormalized />
      <MacroDefinitionsPerRepoLinesNormalized />
      <MacroInvocationsPerCrate />
      <MacroDefinitionsPerCrate />
      <MacroInvocationsPerCrateLinesNormalized />
      <MacroDefinitionsPerCrateLinesNormalized />
      <LinesPerRepo />
    </div>
  );
}
