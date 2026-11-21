export {};

const childProcessModule = "node:child_process";
const { spawnSync } = await import(childProcessModule);

function run(command: string, args: string[]): void {
  const result = spawnSync(command, args, { stdio: "inherit", shell: false });
  if (result.error) throw result.error;
  if (result.status !== 0) process.exit(result.status ?? 1);
}

const staged = spawnSync("git", ["diff", "--cached", "--name-only", "--diff-filter=ACMR", "-z"], {
  encoding: "utf8",
});
if (staged.error) throw staged.error;
if (staged.status !== 0) process.exit(staged.status ?? 1);

const files: string[] = (staged.stdout as string)
  .split("\0")
  .filter((file: string) => /\.(?:ts|tsx)$/i.test(file));

if (files.length > 0) {
  run("bun", ["x", "oxfmt", "--check", ...files]);
  run("bun", ["x", "oxlint", "--deny-warnings", ...files]);
}

run("nsh", ["check"]);
