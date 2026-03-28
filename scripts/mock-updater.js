import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import { execFileSync } from 'child_process';

/**
 * Tauri 本地更新模拟脚本
 * 功能：
 * 1. 默认生成一个仅用于 check() 检测的 mock-server/latest.json
 * 2. 可选地复用已构建的 MSI 生成可下载的本地包
 * 3. 提示如何启动本地服务器进行测试
 */

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const rootDir = path.resolve(__dirname, '..');
const mockDir = path.join(rootDir, 'mock-server');
const tauriConfPath = path.join(rootDir, 'src-tauri/tauri.conf.json5');
const bundleDir = path.join(rootDir, 'src-tauri/target/release/bundle/msi');
const args = process.argv.slice(2);
const shouldUseBuiltInstaller = args.includes('--use-built-installer');

function getNextVersion(currentVersion) {
  const [major, minor, patch] = currentVersion.split('.').map(Number);

  if ([major, minor, patch].some(Number.isNaN)) {
    throw new Error(`无法从当前版本号推导 mock 版本: ${currentVersion}`);
  }

  return `${major}.${minor}.${patch + 1}`;
}

function resolveBuiltInstaller() {
  if (!fs.existsSync(bundleDir)) {
    throw new Error(`未找到构建目录: ${bundleDir}`);
  }

  const files = fs.readdirSync(bundleDir);
  const msiFile = files.find((file) => file.endsWith('.msi') && !file.includes('installer'));

  if (!msiFile) {
    throw new Error('在 bundle 目录中未找到 .msi 文件');
  }

  return {
    name: msiFile,
    path: path.join(bundleDir, msiFile),
  };
}

function signInstaller(installerPath) {
  try {
    const output = execFileSync(
      'npx',
      ['tauri', 'signer', 'sign', installerPath],
      {
        cwd: rootDir,
        encoding: 'utf-8',
        env: process.env,
      }
    );
    const lines = output.split(/\r?\n/);
    const signature = lines.find((line) => line.trim().startsWith('dW50cnVzdGVk'))?.trim();

    if (!signature) {
      throw new Error('未能从 tauri signer 输出中提取签名');
    }

    return signature;
  } catch (error) {
    throw new Error(`签名失败: ${error.message}`);
  }
}

async function run() {
  console.log('🚀 开始准备本地更新测试环境...');

  if (!fs.existsSync(tauriConfPath)) {
    console.error('❌ 未找到 tauri.conf.json5');
    return;
  }

  const tauriConf = JSON.parse(fs.readFileSync(tauriConfPath, 'utf-8'));
  const currentVersion = tauriConf.version;
  const productName = tauriConf.productName || 'CodePulse';
  const nextVersion = getNextVersion(currentVersion);
  const targetInstallerName = `${productName}_${nextVersion}_x64_en-US.msi`;
  const targetInstallerPath = path.join(mockDir, targetInstallerName);
  const latestJsonPath = path.join(mockDir, 'latest.json');
  const defaultDownloadUrl = `http://localhost:8080/${targetInstallerName}`;
  const latestJson = {
    version: nextVersion,
    notes: `Local mock update for detection only. Current: ${currentVersion}, Target: ${nextVersion}`,
    pub_date: new Date().toISOString(),
    platforms: {
      'windows-x86_64': {
        signature: 'LOCAL_CHECK_ONLY_SIGNATURE',
        url: defaultDownloadUrl,
      },
    },
  };

  console.log(`[Info] 当前版本: ${currentVersion}, 模拟目标版本: ${nextVersion}`);
  console.log(`[Info] 当前模式: ${shouldUseBuiltInstaller ? '复用已构建安装包' : '仅检测更新'}`);

  if (!fs.existsSync(mockDir)) {
    fs.mkdirSync(mockDir, { recursive: true });
  }

  if (shouldUseBuiltInstaller) {
    if (!process.env.TAURI_SIGNING_PRIVATE_KEY) {
      console.error('❌ 复用已构建安装包模式需要环境变量 TAURI_SIGNING_PRIVATE_KEY');
      console.log('请先在 Powershell 中执行: $env:TAURI_SIGNING_PRIVATE_KEY="你的私钥内容"');
      return;
    }

    let builtInstaller;

    try {
      builtInstaller = resolveBuiltInstaller();
      console.log(`[Info] 找到安装包: ${builtInstaller.name}`);
      console.log('[Task] 正在生成签名...');
      latestJson.platforms['windows-x86_64'].signature = signInstaller(builtInstaller.path);
      fs.copyFileSync(builtInstaller.path, targetInstallerPath);
      latestJson.notes = `Local mock update with built installer. Current: ${currentVersion}, Target: ${nextVersion}`;
      console.log(`[Task] 已将安装包拷贝并重命名为: ${targetInstallerName}`);
    } catch (error) {
      console.error(`❌ ${error.message}`);
      console.log('请先执行 npx tauri build，或去掉 --use-built-installer 改为仅检测更新模式。');
      return;
    }
  } else {
    latestJson.notes = `Local mock update for detection only. Current: ${currentVersion}, Target: ${nextVersion}. Please cancel installation after check.`;
    console.log('[Warn] 当前 latest.json 仅用于 check() 检测。若继续安装，预期会失败。');
  }

  fs.writeFileSync(latestJsonPath, JSON.stringify(latestJson, null, 2) + '\n');
  console.log('✅ 已生成 mock-server/latest.json');

  console.log('\n' + '='.repeat(50));
  console.log('🎉 本地测试环境准备就绪！');
  console.log('1. 启动本地服务器:');
  console.log('   npx serve ./mock-server -p 8080');
  console.log('\n2. 临时修改 src-tauri/tauri.conf.json5:');
  console.log(`   "endpoints": ["http://localhost:8080/latest.json"]`);
  console.log('\n3. 启动 Tauri 应用测试更新检测:');
  console.log('   npm run tauri -- dev');
  if (!shouldUseBuiltInstaller) {
    console.log('\n4. 当前为仅检测模式:');
    console.log('   建议只验证是否能发现新版本，不要继续安装。');
    console.log('   如需测试下载链路，请先构建安装包后执行: npm run mock-updater -- --use-built-installer');
  }
  console.log('='.repeat(50));
}

run();
