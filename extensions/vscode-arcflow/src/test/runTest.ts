import * as fs from "fs";
import * as os from "os";
import * as path from "path";
import { runTests } from "@vscode/test-electron";

function resolveLocalVsCode(): string | undefined {
  const candidates = [
    path.join(os.homedir(), "AppData", "Local", "Programs", "Microsoft VS Code", "Code.exe"),
    path.join(os.homedir(), "AppData", "Local", "Programs", "cursor", "Cursor.exe"),
    "/usr/share/code/code",
    "/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code",
  ];
  return candidates.find((candidate) => fs.existsSync(candidate));
}

async function main(): Promise<void> {
  const extensionDevelopmentPath = path.resolve(__dirname, "../../");
  const extensionTestsPath = path.resolve(__dirname, "./suite/index.js");
  const vscodeExecutablePath = resolveLocalVsCode();

  await runTests({
    extensionDevelopmentPath,
    extensionTestsPath,
    ...(vscodeExecutablePath ? { vscodeExecutablePath } : {}),
  });
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
