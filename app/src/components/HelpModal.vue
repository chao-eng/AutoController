<script setup lang="ts">
import { Gamepad2, Circle, FileCode2, CalendarClock, Settings, Monitor } from '@lucide/vue'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from '@/components/ui/dialog'
import { ScrollArea } from '@/components/ui/scroll-area'

const emit = defineEmits<{
  close: []
}>()
</script>

<template>
  <Dialog :open="true" @update:open="(v) => { if (!v) emit('close') }">
    <DialogContent class="sm:max-w-[580px] max-h-[85vh]">
      <DialogHeader>
        <DialogTitle>使用说明</DialogTitle>
        <DialogDescription>
          AutoController 的完整功能指南与快速上手教程
        </DialogDescription>
      </DialogHeader>
      <ScrollArea class="max-h-[65vh] pr-4">
        <div class="flex flex-col gap-4">
          <section class="rounded-lg border border-border bg-card p-4">
            <div class="flex items-center gap-2 mb-2 text-primary">
              <Monitor :size="18" />
              <h3 class="text-sm font-semibold text-primary">设备监控</h3>
            </div>
            <ul class="list-disc pl-5 flex flex-col gap-1">
              <li class="text-sm text-muted-foreground leading-relaxed">点击「添加设备」创建虚拟 Xbox 360 手柄</li>
              <li class="text-sm text-muted-foreground leading-relaxed">点击设备卡片可选中并查看实时输入状态</li>
              <li class="text-sm text-muted-foreground leading-relaxed">选中设备后可通过摇杆可视化和扳机条查看手柄状态</li>
              <li class="text-sm text-muted-foreground leading-relaxed">最多可同时创建 8 个虚拟设备</li>
              <li class="text-sm text-muted-foreground leading-relaxed">设备编号从 0 开始，脚本中可用数字编号引用设备</li>
              <li class="text-sm text-muted-foreground leading-relaxed">绿色图标 = ViGEmBus 已连接（系统可识别），黄色图标 = 模拟模式</li>
            </ul>
          </section>

          <section class="rounded-lg border border-border bg-card p-4">
            <div class="flex items-center gap-2 mb-2 text-primary">
              <Monitor :size="18" />
              <h3 class="text-sm font-semibold text-primary">ViGEmBus 驱动安装</h3>
            </div>
            <ul class="list-disc pl-5 flex flex-col gap-1">
              <li class="text-sm text-muted-foreground leading-relaxed">虚拟手柄需要 ViGEmBus 驱动才能被系统识别</li>
              <li class="text-sm text-muted-foreground leading-relaxed">1. 下载安装 ViGEmBus 驱动：github.com/nefarius/ViGEmBus/releases</li>
              <li class="text-sm text-muted-foreground leading-relaxed">2. 将 ViGEmClient.dll 放到程序同目录下</li>
              <li class="text-sm text-muted-foreground leading-relaxed">3. 重启程序，设备监控页面应显示「ViGEmBus 已连接」</li>
              <li class="text-sm text-muted-foreground leading-relaxed">安装后 Xbox Accessories 应能检测到虚拟手柄</li>
            </ul>
          </section>

          <section class="rounded-lg border border-border bg-card p-4">
            <div class="flex items-center gap-2 mb-2 text-primary">
              <Gamepad2 :size="18" />
              <h3 class="text-sm font-semibold text-primary">宏控制</h3>
            </div>
            <ul class="list-disc pl-5 flex flex-col gap-1">
              <li class="text-sm text-muted-foreground leading-relaxed">输入宏名称后点击「开始录制」记录手柄操作</li>
              <li class="text-sm text-muted-foreground leading-relaxed">录制完成后点击「停止录制」保存宏</li>
              <li class="text-sm text-muted-foreground leading-relaxed">可通过回放速度滑块调整播放速率（50%~200%）</li>
              <li class="text-sm text-muted-foreground leading-relaxed">设置循环次数可让宏重复执行</li>
              <li class="text-sm text-muted-foreground leading-relaxed">点击播放按钮回放宏，点击删除按钮移除宏</li>
            </ul>
          </section>

          <section class="rounded-lg border border-border bg-card p-4">
            <div class="flex items-center gap-2 mb-2 text-primary">
              <FileCode2 :size="18" />
              <h3 class="text-sm font-semibold text-primary">脚本编辑器</h3>
            </div>
            <ul class="list-disc pl-5 flex flex-col gap-1">
              <li class="text-sm text-muted-foreground leading-relaxed">左侧面板管理脚本列表，输入名称后点击「新建」创建脚本</li>
              <li class="text-sm text-muted-foreground leading-relaxed">中间区域为代码编辑器，支持手柄控制和延时等 API</li>
              <li class="text-sm text-muted-foreground leading-relaxed">右侧面板提供 API 参考文档</li>
              <li class="text-sm text-muted-foreground leading-relaxed">点击「保存」保存脚本，点击「运行」执行当前脚本</li>
              <li class="text-sm text-muted-foreground leading-relaxed">脚本 API 示例：<code class="rounded bg-muted px-1.5 py-0.5 font-mono text-xs text-foreground">controller.press(id, "A")</code> 按下按键，<code class="rounded bg-muted px-1.5 py-0.5 font-mono text-xs text-foreground">await timing.sleep(1000)</code> 等待 1 秒</li>
            </ul>
          </section>

          <section class="rounded-lg border border-border bg-card p-4">
            <div class="flex items-center gap-2 mb-2 text-primary">
              <CalendarClock :size="18" />
              <h3 class="text-sm font-semibold text-primary">任务调度</h3>
            </div>
            <ul class="list-disc pl-5 flex flex-col gap-1">
              <li class="text-sm text-muted-foreground leading-relaxed">创建定时任务可自动执行宏或脚本</li>
              <li class="text-sm text-muted-foreground leading-relaxed">支持单次执行、每日定时、固定间隔和 Cron 表达式四种调度方式</li>
              <li class="text-sm text-muted-foreground leading-relaxed">可通过开关按钮启用或禁用任务</li>
              <li class="text-sm text-muted-foreground leading-relaxed">每个任务可设置优先级，高优先级任务优先执行</li>
            </ul>
          </section>

          <section class="rounded-lg border border-border bg-card p-4">
            <div class="flex items-center gap-2 mb-2 text-primary">
              <Settings :size="18" />
              <h3 class="text-sm font-semibold text-primary">参数配置</h3>
            </div>
            <ul class="list-disc pl-5 flex flex-col gap-1">
              <li class="text-sm text-muted-foreground leading-relaxed">通用设置：开机自启动、最小化到托盘、日志级别</li>
              <li class="text-sm text-muted-foreground leading-relaxed">Profile 管理：为不同游戏创建配置文件，绑定游戏进程</li>
              <li class="text-sm text-muted-foreground leading-relaxed">配置修改后自动保存</li>
            </ul>
          </section>

          <section class="rounded-lg border border-border bg-card p-4">
            <div class="flex items-center gap-2 mb-2 text-primary">
              <Circle :size="18" />
              <h3 class="text-sm font-semibold text-primary">快速上手</h3>
            </div>
            <ol class="list-decimal pl-5 flex flex-col gap-1">
              <li class="text-sm text-muted-foreground leading-relaxed">安装 ViGEmBus 驱动（首次使用）</li>
              <li class="text-sm text-muted-foreground leading-relaxed">在「设备监控」页面添加虚拟手柄</li>
              <li class="text-sm text-muted-foreground leading-relaxed">在「宏控制」页面录制手柄操作，或在「脚本编辑器」编写自动化脚本</li>
              <li class="text-sm text-muted-foreground leading-relaxed">使用「任务调度」设置定时自动执行</li>
              <li class="text-sm text-muted-foreground leading-relaxed">在「日志查看」页面排查问题</li>
            </ol>
          </section>
        </div>
      </ScrollArea>
    </DialogContent>
  </Dialog>
</template>