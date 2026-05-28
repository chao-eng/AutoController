<script setup lang="ts">
import { X, Gamepad2, Circle, FileCode2, CalendarClock, Settings, Monitor } from '@lucide/vue'

const emit = defineEmits<{
  close: []
}>()
</script>

<template>
  <div class="modal-overlay" @click.self="emit('close')">
    <div class="help-modal">
      <div class="modal-header">
        <h2>使用说明</h2>
        <button class="close-btn" @click="emit('close')">
          <X :size="18" />
        </button>
      </div>

      <div class="modal-body">
        <section class="help-section">
          <div class="section-title">
            <Monitor :size="18" />
            <h3>设备监控</h3>
          </div>
          <ul>
            <li>点击「添加设备」创建虚拟 Xbox 360 手柄</li>
            <li>点击设备卡片可选中并查看实时输入状态</li>
            <li>选中设备后可通过摇杆可视化和扳机条查看手柄状态</li>
            <li>最多可同时创建 8 个虚拟设备</li>
            <li>设备编号从 0 开始，脚本中可用数字编号引用设备</li>
            <li>绿色图标 = ViGEmBus 已连接（系统可识别），黄色图标 = 模拟模式</li>
          </ul>
        </section>

        <section class="help-section">
          <div class="section-title">
            <Monitor :size="18" />
            <h3>ViGEmBus 驱动安装</h3>
          </div>
          <ul>
            <li>虚拟手柄需要 ViGEmBus 驱动才能被系统识别</li>
            <li>1. 下载安装 ViGEmBus 驱动：github.com/nefarius/ViGEmBus/releases</li>
            <li>2. 将 ViGEmClient.dll 放到程序同目录下</li>
            <li>3. 重启程序，设备监控页面应显示「ViGEmBus 已连接」</li>
            <li>安装后 Xbox Accessories 应能检测到虚拟手柄</li>
          </ul>
        </section>

        <section class="help-section">
          <div class="section-title">
            <Gamepad2 :size="18" />
            <h3>宏控制</h3>
          </div>
          <ul>
            <li>输入宏名称后点击「开始录制」记录手柄操作</li>
            <li>录制完成后点击「停止录制」保存宏</li>
            <li>可通过回放速度滑块调整播放速率（50%~200%）</li>
            <li>设置循环次数可让宏重复执行</li>
            <li>点击播放按钮回放宏，点击删除按钮移除宏</li>
          </ul>
        </section>

        <section class="help-section">
          <div class="section-title">
            <FileCode2 :size="18" />
            <h3>脚本编辑器</h3>
          </div>
          <ul>
            <li>左侧面板管理脚本列表，输入名称后点击「新建」创建脚本</li>
            <li>中间区域为代码编辑器，支持手柄控制和延时等 API</li>
            <li>右侧面板提供 API 参考文档</li>
            <li>点击「保存」保存脚本，点击「运行」执行当前脚本</li>
            <li>脚本 API 示例：<code>controller.press(id, "A")</code> 按下按键，<code>await timing.sleep(1000)</code> 等待 1 秒</li>
          </ul>
        </section>

        <section class="help-section">
          <div class="section-title">
            <CalendarClock :size="18" />
            <h3>任务调度</h3>
          </div>
          <ul>
            <li>创建定时任务可自动执行宏或脚本</li>
            <li>支持单次执行、每日定时、固定间隔和 Cron 表达式四种调度方式</li>
            <li>可通过开关按钮启用或禁用任务</li>
            <li>每个任务可设置优先级，高优先级任务优先执行</li>
          </ul>
        </section>

        <section class="help-section">
          <div class="section-title">
            <Settings :size="18" />
            <h3>参数配置</h3>
          </div>
          <ul>
            <li>通用设置：开机自启动、最小化到托盘、日志级别</li>
            <li>Profile 管理：为不同游戏创建配置文件，绑定游戏进程</li>
            <li>配置修改后自动保存</li>
          </ul>
        </section>

        <section class="help-section">
          <div class="section-title">
            <Circle :size="18" />
            <h3>快速上手</h3>
          </div>
          <ol>
            <li>安装 ViGEmBus 驱动（首次使用）</li>
            <li>在「设备监控」页面添加虚拟手柄</li>
            <li>在「宏控制」页面录制手柄操作，或在「脚本编辑器」编写自动化脚本</li>
            <li>使用「任务调度」设置定时自动执行</li>
            <li>在「日志查看」页面排查问题</li>
          </ol>
        </section>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  backdrop-filter: blur(4px);
}

.help-modal {
  width: 560px;
  max-height: 80vh;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-xl);
  display: flex;
  flex-direction: column;
  box-shadow: var(--shadow-lg);
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--space-md) var(--space-lg);
  border-bottom: 1px solid var(--color-border);
  flex-shrink: 0;
}

.modal-header h2 {
  font-size: 16px;
  font-weight: 600;
}

.close-btn {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-sm);
  color: var(--color-text-dim);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.close-btn:hover {
  background: var(--color-surface-elevated);
  color: var(--color-text);
}

.modal-body {
  padding: var(--space-lg);
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: var(--space-lg);
}

.help-section {
  background: var(--color-surface-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  padding: var(--space-md);
}

.section-title {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  margin-bottom: var(--space-sm);
  color: var(--color-cta);
}

.section-title h3 {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-cta);
}

.help-section ul,
.help-section ol {
  padding-left: var(--space-lg);
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
}

.help-section li {
  font-size: 13px;
  color: var(--color-text-muted);
  line-height: 1.6;
}

.help-section code {
  font-family: var(--font-heading);
  font-size: 12px;
  background: var(--color-background);
  padding: 1px 6px;
  border-radius: var(--radius-sm);
  color: var(--color-info);
}

.help-section ol li::marker {
  color: var(--color-cta);
  font-weight: 600;
}

.help-section ul li::marker {
  color: var(--color-cta);
}
</style>
