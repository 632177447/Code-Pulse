import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import { applyEdits, modify } from 'jsonc-parser';
import { execSync } from 'child_process';

/**
 * Tauri 版本同步与自动发布脚本
 * 使用方式: 
 * 1. 自动同步 package.json 的版本: npm run release
 * 2. 指定版本号: npm run release 2.0.0
 * 3. 仅同步配置文件版本号(不提交/Tag): npm run release sync 2.0.0
 */

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const rootDir = path.resolve(__dirname, '..');

const paths = {
  packageJson: path.join(rootDir, 'package.json'),
  tauriConf: path.join(rootDir, 'src-tauri/tauri.conf.json5'),
  cargoToml: path.join(rootDir, 'src-tauri/Cargo.toml'),
};

// 执行 Shell 命令的辅助函数
function runCommand(command) {
  try {
    console.log(`[Exec] ${command}`);
    execSync(command, { stdio: 'inherit', cwd: rootDir });
  } catch (error) {
    console.error(`❌ 执行命令失败: ${command}`);
    process.exit(1);
  }
}

// 获取参数并清理
const rawArgs = process.argv.slice(2);
const keywords = ['sync', '-s', '--sync'];

const args = rawArgs.map(arg => arg.trim());
// 宽松匹配标志位
const syncOnly = 
  process.argv.some(a => keywords.some(k => a.toLowerCase().includes(k))) || 
  process.env.npm_config_sync === 'true' ||
  args.some(arg => keywords.includes(arg.toLowerCase()));

// 获取目标版本号 (排除以 - 开头和关键字)
let targetVersion = args.find(arg => !arg.startsWith('-') && !keywords.includes(arg.toLowerCase()));

// 如果没有指定版本，则读取 package.json 的版本作为基准
if (!targetVersion) {
  const pkg = JSON.parse(fs.readFileSync(paths.packageJson, 'utf8'));
  targetVersion = pkg.version;
  console.log(`[Release] 未指定版本号，将使用 package.json 中的版本: ${targetVersion}`);
}

const vTag = `v${targetVersion}`;

function updateVersion() {
  console.log(`[Release] 1. 开始同步版本号至: ${targetVersion}...`);

  // 1. 同步 package.json
  const pkg = JSON.parse(fs.readFileSync(paths.packageJson, 'utf8'));
  pkg.version = targetVersion;
  fs.writeFileSync(paths.packageJson, JSON.stringify(pkg, null, 2) + '\n');
  console.log('✅ Updated package.json');

  // 2. 同步 tauri.conf.json5
  const tauriConfContent = fs.readFileSync(paths.tauriConf, 'utf8');
  const edits = modify(tauriConfContent, ['version'], targetVersion, {
    formattingOptions: {
      insertSpaces: true,
      tabSize: 2
    }
  });
  const updatedTauriConf = applyEdits(tauriConfContent, edits);
  fs.writeFileSync(paths.tauriConf, updatedTauriConf);
  console.log('✅ Updated tauri.conf.json5');

  // 3. 同步 Cargo.toml
  let cargoContent = fs.readFileSync(paths.cargoToml, 'utf8');
  cargoContent = cargoContent.replace(
    /^version = ".*"$/m,
    `version = "${targetVersion}"`
  );
  fs.writeFileSync(paths.cargoToml, cargoContent);
  console.log('✅ Updated Cargo.toml');

  // 4. 同步 Cargo.lock (确保版本号变更同步到 lock 文件，否则 git add . 无法捕获变化)
  runCommand(`cargo update --manifest-path "${paths.cargoToml}" --offline -p codepulse`);
  console.log('✅ Updated Cargo.lock');

  if (syncOnly) {
    console.log(`\n🚀 版本提示: 配置文件版本号已同步至 ${targetVersion}。开启了 --sync 模式，跳过 Git 操作。`);
    return;
  }

  console.log(`\n[Release] 2. 开始执行 Git 操作...`);

  // 获取当前分支名称
  const currentBranch = execSync('git rev-parse --abbrev-ref HEAD', { encoding: 'utf8' }).trim();

  // 执行 Git 命令
  runCommand(`git add .`);
  
  // 检查是否有变动需要提交
  const status = execSync('git status --porcelain', { encoding: 'utf8' }).trim();
  if (status) {
    runCommand(`git commit -m "${vTag}"`);
  } else {
    console.log('[Release] 工作区干净，跳过 commit。');
  }
  
  // 检查 Tag 是否已存在，如果存在则删除
  try {
    execSync(`git tag -d ${vTag}`, { stdio: 'ignore' });
    execSync(`git push origin :refs/tags/${vTag}`, { stdio: 'ignore' });
  } catch (e) {}

  runCommand(`git tag ${vTag}`);
  runCommand(`git push origin ${currentBranch}`);
  runCommand(`git push origin ${vTag}`);

  console.log(`\n🚀 发布流水线完成！版本 ${vTag} 已推送至 GitHub。`);
}

updateVersion();
