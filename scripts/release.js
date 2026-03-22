import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

/**
 * Tauri 版本同步脚本
 * 使用方式: 
 * 1. 自动同步 package.json 的版本: npm run release
 * 2. 指定版本号: npm run release 1.2.0
 */

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const rootDir = path.resolve(__dirname, '..');

const paths = {
  packageJson: path.join(rootDir, 'package.json'),
  tauriConf: path.join(rootDir, 'src-tauri/tauri.conf.json'),
  cargoToml: path.join(rootDir, 'src-tauri/Cargo.toml'),
};

// 获取目标版本号
let targetVersion = process.argv[2];

// 如果没有传参，则读取 package.json 的版本作为基准
if (!targetVersion) {
  const pkg = JSON.parse(fs.readFileSync(paths.packageJson, 'utf8'));
  targetVersion = pkg.version;
  console.log(`[Release] 未指定版本，将使用 package.json 中的版本号: ${targetVersion}`);
}

function updateVersion() {
  console.log(`[Release] 开始同步版本号至: ${targetVersion}...`);

  // 1. 同步 package.json
  const pkg = JSON.parse(fs.readFileSync(paths.packageJson, 'utf8'));
  pkg.version = targetVersion;
  fs.writeFileSync(paths.packageJson, JSON.stringify(pkg, null, 2) + '\n');
  console.log('✅ Updated package.json');

  // 2. 同步 tauri.conf.json
  const tauriConf = JSON.parse(fs.readFileSync(paths.tauriConf, 'utf8'));
  tauriConf.version = targetVersion;
  fs.writeFileSync(paths.tauriConf, JSON.stringify(tauriConf, null, 2) + '\n');
  console.log('✅ Updated tauri.conf.json');

  // 3. 同步 Cargo.toml (使用正则，防止破坏文件其他部分)
  let cargoContent = fs.readFileSync(paths.cargoToml, 'utf8');
  cargoContent = cargoContent.replace(
    /^version = ".*"$/m,
    `version = "${targetVersion}"`
  );
  fs.writeFileSync(paths.cargoToml, cargoContent);
  console.log('✅ Updated Cargo.toml');

  console.log('\n🚀 版本同步任务完成！请记得提交代码并打 Tag。');
}

updateVersion();
