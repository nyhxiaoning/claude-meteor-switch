# 重置应用图标指南

## 步骤

### 1. 停止开发服务器
- 在运行 `pnpm tauri dev` 的终端按 `Ctrl + C`

### 2. 完全退出应用
- 右键点击 Dock 中的应用图标 → 退出

### 3. 清理构建缓存
```bash
# 清理构建产物
rm -rf src-tauri/target/
rm -rf dist/
```

### 4. 清理 macOS Dock 缓存（可选，但推荐）
```bash
# 重置 Dock
killall Dock

# 如果还不行，清理缓存
rm ~/Library/Caches/com.apple.dock.iconcache -f
killall Dock
```

### 5. 重新运行
```bash
pnpm tauri dev
```

## 说明

开发模式下的图标可能被缓存了。完全清理后重新启动应该就能看到新的绿色图标了。

新图标特点：
- 绿色主题背景 (#00DC82)
- 白色 "M" 字母
- 圆角正方形设计
