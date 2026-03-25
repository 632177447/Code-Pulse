import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import { execSync } from 'child_process';

/**
 * Tauri 本地更新模拟脚本
 * 功能：
 * 1. 在本地生成一个版本号更高的 mock-server/latest.json
 * 2. 自动寻找已构建的安装包并生成签名
 * 3. 提示如何启动本地服务器进行测试
 */

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const rootDir = path.resolve(__dirname, '..');
const mockDir = path.join(rootDir, 'mock-server');

// 配置路径
const tauriConfPath = path.join(rootDir, 'src-tauri/tauri.conf.json');

async function run() {
  console.log('🚀 开始准备本地更新测试环境...');

  // 1. 读取当前配置
  if (!fs.existsSync(tauriConfPath)) {
    console.error('❌ 未找到 tauri.conf.json');
    return;
  }
  const tauriConf = JSON.parse(fs.readFileSync(tauriConfPath, 'utf-8'));
  const currentVersion = tauriConf.version;
  const productName = tauriConf.productName || 'CodePulse';
  
  // 模拟一个更高的版本
  const [major, minor, patch] = currentVersion.split('.').map(Number);
  const nextVersion = `${major}.${minor}.${patch + 1}`;

  console.log(`[Info] 当前版本: ${currentVersion}, 模拟目标版本: ${nextVersion}`);

  // 2. 检查私钥环境变量
  const privateKey = process.env.TAURI_SIGNING_PRIVATE_KEY;
  if (!privateKey) {
    console.warn('⚠️ 未检测到环境变量 TAURI_SIGNING_PRIVATE_KEY');
    console.log('请先执行以下命令设置私钥（或将其写入 .env）:');
    console.log('Windows (Powershell): $env:TAURI_SIGNING_PRIVATE_KEY="你的私钥内容"');
    console.log('\n如果你没有私钥，可以生成一个测试用的：npx tauri signer generate -w ./test.key');
    return;
  }

  // 3. 寻找安装包 (优先找 MSI)
  const bundleDir = path.join(rootDir, `src-tauri/target/release/bundle/msi`);
  if (!fs.existsSync(bundleDir)) {
    console.error(`❌ 未找到构建目录: ${bundleDir}`);
    console.log('请先运行 npm run tauri build 进行一次正式构建。');
    return;
  }

  const files = fs.readdirSync(bundleDir);
  const msiFile = files.find(f => f.endsWith('.msi') && !f.includes('installer'));
  
  if (!msiFile) {
    console.error('❌ 在 bundle 目录中未找到 .msi 文件');
    return;
  }

  const msiPath = path.join(bundleDir, msiFile);
  console.log(`[Info] 找到安装包: ${msiFile}`);

  // 4. 生成签名
  console.log('[Task] 正在生成签名...');
  let signature = '';
  try {
    // 使用 tauri-cli 生成签名
    const output = execSync(`npx tauri signer sign -k "${privateKey}" "${msiPath}"`, { encoding: 'utf-8' });
    // 提取输出中以 dW50cnVzdGVk 开头的签名字符串
    const lines = output.split('\n');
    signature = lines.find(line => line.trim().startsWith('dW50cnVzdGVk'))?.trim();
    
    if (!signature) {
      // 备选方案：尝试提取非空最后一行
      signature = lines.filter(l => l.trim()).pop()?.trim();
    }
    
    console.log(`✅ 签名提取成功: ${signature.substring(0, 20)}...`);
  } catch (err) {
    console.error('❌ 签名失败:', err.message);
    return;
  }

  // 5. 准备 mock-server 目录
  if (!fs.existsSync(mockDir)) {
    fs.mkdirSync(mockDir);
  }

  // 拷贝安装包到 mock-server (为了让本地 HTTP server 访问到)
  const targetMsiName = `${productName}_${nextVersion}_x64_en-US.msi`;
  fs.copyFileSync(msiPath, path.join(mockDir, targetMsiName));
  console.log(`[Task] 已将安装包拷贝并重命名为: ${targetMsiName}`);

  // 6. 生成最新的 latest.json (符合 Tauri v2 格式)
  const latestJson = {
    version: nextVersion,
    notes: `Test update from ${currentVersion} to ${nextVersion}`,
    pub_date: new Date().toISOString(),
    platforms: {
      "windows-x86_64": {
        signature: signature,
        url: `http://localhost:8080/${targetMsiName}`
      }
    }
  };

  fs.writeFileSync(path.join(mockDir, 'latest.json'), JSON.stringify(latestJson, null, 2));
  console.log('✅ 已生成 mock-server/latest.json');

  console.log('\n' + '='.repeat(50));
  console.log('🎉 本地测试环境准备就绪！');
  console.log('1. 启动本地服务器:');
  console.log('   npx serve ./mock-server -p 8080');
  console.log('\n2. 临时修改 src-tauri/tauri.conf.json:');
  console.log(`   "endpoints": ["http://localhost:8080/latest.json"]`);
  console.log('\n3. 启动开发环境测试更新流程:');
  console.log('   npm run tauri dev');
  console.log('='.repeat(50));
}

run();
