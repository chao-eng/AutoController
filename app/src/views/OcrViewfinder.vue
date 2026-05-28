<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

let appWindow: any = null

const winX = ref(0)
const winY = ref(0)
const winWidth = ref(400)
const winHeight = ref(150)

// 刷新并获取当前的物理像素坐标与视口大小
async function updateCoordinates() {
  try {
    if (!appWindow) return
    const pos = await appWindow.outerPosition()
    const size = await appWindow.innerSize()
    winX.value = pos.x
    winY.value = pos.y
    winWidth.value = size.width
    winHeight.value = size.height
  } catch (e) {
    console.error('Failed to get window coordinates:', e)
  }
}

let updateInterval: number | null = null

onMounted(async () => {
  // 动态载入 Webview API，确保顶级安全隔离，绝不崩溃主窗口
  const { getCurrentWindow } = await import('@tauri-apps/api/window')
  appWindow = getCurrentWindow()

  // 核心：设置极为微弱的半透明背景（1% 不透明），防止 Windows 穿透鼠标点击事件，使得拖拽与点击正常响应
  document.body.style.background = 'rgba(0, 0, 0, 0.01)'
  document.documentElement.style.background = 'rgba(0, 0, 0, 0.01)'
  
  await updateCoordinates()
  
  // 定期轮询（每 100 毫秒）更新实时物理分辨率
  updateInterval = window.setInterval(updateCoordinates, 100)
})

onUnmounted(() => {
  // 恢复全局 body 默认背景色
  document.body.style.background = ''
  document.documentElement.style.background = ''
  
  if (updateInterval) {
    clearInterval(updateInterval)
  }
})

// 取消框选，直接关闭窗口
async function handleCancel() {
  if (appWindow) {
    await appWindow.close()
  }
}

// 确认标定：直接调用 Rust 后端 save_ocr_region 命令，保存当前坐标并广播给主窗口刷新
async function handleConfirm() {
  await updateCoordinates()
  
  try {
    await invoke('save_ocr_region', {
      x: winX.value,
      y: winY.value,
      w: winWidth.value,
      h: winHeight.value
    })
  } catch (e) {
    console.error('Failed to save OCR region coordinates:', e)
  }
  
  // 关闭标定框窗口
  if (appWindow) {
    await appWindow.close()
  }
}
</script>

<template>
  <div class="viewfinder-container">
    <!-- 四角霓虹质感标定框线 -->
    <div class="corner top-left"></div>
    <div class="corner top-right"></div>
    <div class="corner bottom-left"></div>
    <div class="corner bottom-right"></div>

    <!-- 中间全透明点击与拖动拖拽手柄区域，使用 data-tauri-drag-region 属性使其能整体拖拽移动 -->
    <div class="viewfinder-target" data-tauri-drag-region>
      <div class="target-crosshair" data-tauri-drag-region></div>
      <div class="target-hint" data-tauri-drag-region>
        ◀ 按住并拖拽此框体中央以移动 ｜ 拖拽四边缘以缩放尺寸 ▶
      </div>
    </div>

    <!-- 底部悬浮控制台面板 -->
    <div class="viewfinder-controls">
      <div class="controls-status">
        <span class="status-pulse"></span>
        <span class="status-text">
          OCR 标定框 ({{ winX }}, {{ winY }}) [{{ winWidth }} × {{ winHeight }} px]
        </span>
      </div>
      <div class="controls-actions">
        <button class="action-btn cancel-btn" @click="handleCancel">取消</button>
        <button class="action-btn confirm-btn" @click="handleConfirm">🎯 确定标定</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* 整个视口背景使用极低不透明度，阻断鼠标点击穿透，使其可拖拽与缩放 */
.viewfinder-container {
  width: 100vw;
  height: 100vh;
  background: rgba(0, 0, 0, 0.01) !important;
  position: relative;
  display: flex;
  flex-direction: column;
  box-sizing: border-box;
  padding: 12px;
  overflow: hidden;
  font-family: var(--font-heading, "Segoe UI", sans-serif);
}

/* 四角高亮Glowing绿色括号线 */
.corner {
  position: absolute;
  width: 20px;
  height: 20px;
  border: 3px solid #22c55e;
  filter: drop-shadow(0 0 5px rgba(34, 197, 94, 0.7));
}

.top-left {
  top: 6px;
  left: 6px;
  border-right: none;
  border-bottom: none;
  border-top-left-radius: 6px;
}

.top-right {
  top: 6px;
  right: 6px;
  border-left: none;
  border-bottom: none;
  border-top-right-radius: 6px;
}

.bottom-left {
  bottom: 60px;
  left: 6px;
  border-right: none;
  border-top: none;
  border-bottom-left-radius: 6px;
}

.bottom-right {
  bottom: 60px;
  right: 6px;
  border-left: none;
  border-top: none;
  border-bottom-right-radius: 6px;
}

/* 核心拖动区域：全透明 */
.viewfinder-target {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  background: rgba(34, 197, 94, 0.02); /* 极薄绿光，方便用户感知识别范围 */
  border: 1px dashed rgba(34, 197, 94, 0.3);
  border-radius: 4px;
  position: relative;
  cursor: move;
  margin-bottom: var(--space-md, 12px);
}

.target-crosshair {
  width: 18px;
  height: 18px;
  position: relative;
  opacity: 0.6;
}

.target-crosshair::before,
.target-crosshair::after {
  content: '';
  position: absolute;
  background: #22c55e;
}

/* 十字星 */
.target-crosshair::before {
  width: 100%;
  height: 2px;
  top: 8px;
  left: 0;
}

.target-crosshair::after {
  width: 2px;
  height: 100%;
  left: 8px;
  top: 0;
}

.target-hint {
  font-size: 10px;
  color: #22c55e;
  font-weight: 500;
  margin-top: 6px;
  opacity: 0.8;
  user-select: none;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.9);
}

/* 底部操作面板 */
.viewfinder-controls {
  height: 42px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: rgba(20, 24, 33, 0.95);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 6px;
  padding: 0 12px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.5);
  flex-shrink: 0;
}

.controls-status {
  display: flex;
  align-items: center;
  gap: 8px;
}

/* 呼吸绿点，指示状态活动中 */
.status-pulse {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #22c55e;
  animation: pulse 1.6s infinite alternate;
  box-shadow: 0 0 6px #22c55e;
}

@keyframes pulse {
  0% { transform: scale(0.9); opacity: 0.6; }
  100% { transform: scale(1.15); opacity: 1; }
}

.status-text {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.85);
  font-weight: 600;
}

.controls-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}

.action-btn {
  font-size: 11px;
  font-weight: 600;
  padding: 4px 10px;
  border-radius: 4px;
  border: none;
  cursor: pointer;
  transition: all 0.15s ease;
}

.cancel-btn {
  background: rgba(255, 255, 255, 0.08);
  color: rgba(255, 255, 255, 0.7);
  border: 1px solid rgba(255, 255, 255, 0.05);
}

.cancel-btn:hover {
  background: rgba(239, 68, 68, 0.2);
  color: #ef4444;
  border-color: rgba(239, 68, 68, 0.1);
}

.confirm-btn {
  background: #22c55e;
  color: white;
  filter: drop-shadow(0 2px 4px rgba(34, 197, 94, 0.3));
}

.confirm-btn:hover {
  background: #1ca84f;
  transform: translateY(-0.5px);
}
</style>
