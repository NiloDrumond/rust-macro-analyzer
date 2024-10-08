// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { DeriveUsage } from "./DeriveUsage";
import type { MacroAnalyzis } from "./MacroAnalyzis";

export type Data = { macro_definitions_per_repo: Array<[string, number]>, macro_invocations_per_repo: Array<[string, number]>, macro_definitions_per_crate: Array<[string, number]>, macro_invocations_per_crate: Array<[string, number]>, lines_per_repo: { [key: string]: number }, lines_per_crate: { [key: string]: number }, characters_per_repo: { [key: string]: number }, characters_per_crate: { [key: string]: number }, derive_usage: DeriveUsage, total_macro_usage: MacroAnalyzis, date: string, };
