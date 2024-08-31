export function getRepoName(input: string): string {
  const parts = input.split(".", 2);
  return parts.length > 1 ? input.substring(input.indexOf(".") + 1) : "";
}

export function getCrateName(input: string): string {
  const parsed = input.endsWith("/") ? input.slice(0, input.length - 1) : input;
  const parts = parsed.split("/");
  return parts.length > 1 ? parts[parts.length - 1] : parsed;
}
