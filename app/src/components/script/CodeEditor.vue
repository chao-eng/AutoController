<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, nextTick } from 'vue'
import * as monaco from 'monaco-editor'

// Import Monaco workers using Vite ?worker syntax to allow proper modular builds
import EditorWorker from 'monaco-editor/esm/vs/editor/editor.worker?worker'
import TsWorker from 'monaco-editor/esm/vs/language/typescript/ts.worker?worker'
import JsonWorker from 'monaco-editor/esm/vs/language/json/json.worker?worker'
import CssWorker from 'monaco-editor/esm/vs/language/css/css.worker?worker'
import HtmlWorker from 'monaco-editor/esm/vs/language/html/html.worker?worker'

// Initialize Monaco Environment for Web Workers loading
if (!self.MonacoEnvironment) {
  self.MonacoEnvironment = {
    getWorker(_: any, label: string) {
      if (label === 'json') {
        return new JsonWorker()
      }
      if (label === 'css' || label === 'less' || label === 'scss') {
        return new CssWorker()
      }
      if (label === 'html' || label === 'handlebars' || label === 'razor') {
        return new HtmlWorker()
      }
      if (label === 'typescript' || label === 'javascript') {
        return new TsWorker()
      }
      return new EditorWorker()
    }
  }
}

const props = defineProps<{
  modelValue: string
  activeLine?: number
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void
  (e: 'save'): void
}>()

const containerRef = ref<HTMLElement | null>(null)
let editor: monaco.editor.IStandaloneCodeEditor | null = null
let activeLineDecoration: string[] = []
let isUpdating = false // Guard to prevent infinite reactive updates

// Register Custom Rhai Language and Theme
function setupMonacoRhai() {
  const langId = 'rhai'

  // Register custom language only if not registered yet
  const registeredLanguages = monaco.languages.getLanguages()
  if (!registeredLanguages.some((l) => l.id === langId)) {
    monaco.languages.register({ id: langId })

    // Tokenizer / Syntax Highlighting
    monaco.languages.setMonarchTokensProvider(langId, {
      keywords: [
        'let', 'const', 'fn', 'return', 'if', 'else', 'while', 'for', 'in', 'loop', 'break', 'continue', 'true', 'false'
      ],
      tokenizer: {
        root: [
          [/[a-zA-Z_]\w*/, {
            cases: {
              '@keywords': 'keyword',
              '@default': 'identifier'
            }
          }],
          [/\/\/.*$/, 'comment'],
          [/[{}()\[\]]/, '@brackets'],
          [/\d+/, 'number'],
          [/"([^"\\]|\\.)*"/, 'string'],
        ]
      }
    })

    // Auto-completions (IntelliSense)
    monaco.languages.registerCompletionItemProvider(langId, {
      provideCompletionItems: (model, position) => {
        const word = model.getWordUntilPosition(position)
        const range = {
          startLineNumber: position.lineNumber,
          endLineNumber: position.lineNumber,
          startColumn: word.startColumn,
          endColumn: word.endColumn
        }

        const suggestions: monaco.languages.CompletionItem[] = [
          {
            label: 'set_default_device',
            kind: monaco.languages.CompletionItemKind.Function,
            insertText: 'set_default_device(${1:0});',
            insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
            detail: 'set_default_device(deviceId)',
            documentation: {
              value: '**设置全局默认操控的虚拟手柄编号**\n\n**参数介绍：**\n- `deviceId`: 默认设备编号（数字 `0`, `1`... 或手柄 UUID 字符串）\n\n**应用场景：**\n如果在脚本开头指定了默认手柄，后续调用 `press`、`release`、`set_thumb` 和 `set_trigger` 时均**无需手动输入手柄编号**，系统会自动控制该手柄！'
            },
            range
          },
          {
            label: 'press',
            kind: monaco.languages.CompletionItemKind.Function,
            insertText: 'press("${1:A}");',
            insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
            detail: 'press(button) 或 press(deviceId, button)',
            documentation: {
              value: '**按下指定虚拟手柄的按键**\n\n**两种使用方式：**\n1. `press("A")` - 直接操控全局默认手柄按下按键。\n2. `press(0, "A")` - 控制指定设备编号（如 0 号）的手柄按下按键。\n\n**参数介绍：**\n- `deviceId` (可选): 设备编号\n- `button`: 按键名称，支持：\n  - 常用键：`"A"`, `"B"`, `"X"`, `"Y"`\n  - 肩键与触发键：`"LB"`, `"RB"`, `"LT"`, `"RT"`\n  - 导航键：`"Back"`, `"Start"`, `"Guide"`\n  - 摇杆按键：`"LS"`, `"RS"`\n  - 方向键：`"Up"`, `"Down"`, `"Left"`, `"Right"`'
            },
            range
          },
          {
            label: 'release',
            kind: monaco.languages.CompletionItemKind.Function,
            insertText: 'release("${1:A}");',
            insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
            detail: 'release(button) 或 release(deviceId, button)',
            documentation: {
              value: '**释放指定虚拟手柄的按键**\n\n**两种使用方式：**\n1. `release("A")` - 释放全局默认手柄按键。\n2. `release(0, "A")` - 释放指定设备编号（如 0 号）的手柄按键。\n\n**参数介绍：**\n- `deviceId` (可选): 设备编号\n- `button`: 按键名称'
            },
            range
          },
          {
            label: 'set_thumb',
            kind: monaco.languages.CompletionItemKind.Function,
            insertText: 'set_thumb("${1:LeftX}", ${2:0.0});',
            insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
            detail: 'set_thumb(axis, value) 或 set_thumb(deviceId, axis, value)',
            documentation: {
              value: '**设置摇杆偏转倾斜度**\n\n**两种使用方式：**\n1. `set_thumb("LeftX", 1.0)` - 调整默认手柄摇杆偏转。\n2. `set_thumb(0, "LeftX", 1.0)` - 调整指定手柄的摇杆偏转。\n\n**参数介绍：**\n- `deviceId` (可选): 设备编号\n- `axis`: 摇杆轴向，支持：\n  - 左摇杆：`"LeftX"` (水平轴), `"LeftY"` (垂直轴)\n  - 右摇杆：`"RightX"` (水平轴), `"RightY"` (垂直轴)\n- `value`: 倾斜度数值，范围在 `[-1.0, 1.0]` 区间内，`0.0` 代表中位悬停。'
            },
            range
          },
          {
            label: 'set_trigger',
            kind: monaco.languages.CompletionItemKind.Function,
            insertText: 'set_trigger("${1:Left}", ${2:0.0});',
            insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
            detail: 'set_trigger(side, value) 或 set_trigger(deviceId, side, value)',
            documentation: {
              value: '**设置扳机键深度压力值**\n\n**两种使用方式：**\n1. `set_trigger("Left", 0.5)` - 调整默认手柄的左扳机压力值。\n2. `set_trigger(0, "Left", 0.5)` - 调整指定手柄的左扳机压力值。\n\n**参数介绍：**\n- `deviceId` (可选): 设备编号\n- `side`: 扳机侧向，支持 `"Left"` 或 `"Right"`\n- `value`: 压力幅度数值，范围在 `[0.0, 1.0]` 区间内，`0.0` 代表未按下，`1.0` 代表按满。'
            },
            range
          },
          {
            label: 'sleep',
            kind: monaco.languages.CompletionItemKind.Function,
            insertText: 'sleep(${1:1000});',
            insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
            detail: 'sleep(ms)',
            documentation: {
              value: '**延时等待休眠**\n\n**参数介绍：**\n- `ms`: 阻断休眠的时长，单位为毫秒（1秒 = 1000毫秒）。'
            },
            range
          },
          {
            label: 'log',
            kind: monaco.languages.CompletionItemKind.Function,
            insertText: 'log("${1:message}");',
            insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
            detail: 'log(message)',
            documentation: {
              value: '**输出调试日志**\n\n**参数介绍：**\n- `message`: 日志文本，信息将实时分发显示在下方的系统运行日志与控制台面板中。'
            },
            range
          },
          {
            label: 'ocr',
            kind: monaco.languages.CompletionItemKind.Function,
            insertText: 'ocr();',
            insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
            detail: 'ocr() / ocr(index) / ocr(x, y, w, h)',
            documentation: {
              value: '**内置 OCR 本地文本识别**\n\n**三种使用方式：**\n1. `ocr()` - 识别默认识别区 #1 文本内容。\n2. `ocr(1)` - 识别指定标定区 #1 的文本内容。\n3. `ocr(100, 200, 300, 150)` - 识别屏幕指定坐标区域 `(x, y, w, h)` 内的文本。\n\n**参数介绍：**\n- `index`: 标定区序号\n- `x, y, w, h`: 矩形区域坐标'
            },
            range
          }
        ]
        return { suggestions }
      }
    })

    // Hover details (Hover Tooltips with rich markdown)
    monaco.languages.registerHoverProvider(langId, {
      provideHover: (model, position) => {
        const word = model.getWordAtPosition(position)
        if (!word) return null

        const name = word.word
        let markdownValue = ''

        if (name === 'set_default_device') {
          markdownValue = `**set_default_device(deviceId)** \\
\\
**设置全局默认操控的虚拟手柄编号**
- \`deviceId\`: 默认设备编号（数字 \`0\`，\`1\`... 或手柄 UUID 字符串）

**说明**：在脚本开头指定默认手柄后，后续调用 \`press\`、\`release\` 等均**无需再传手柄编号**。`
        } else if (name === 'press') {
          markdownValue = `**press(button)** \\
**press(deviceId, button)** \\
\\
**模拟按下虚拟手柄的指定按键**
- \`deviceId\` (可选): 设备编号或 UUID
- \`button\`: 按键字符串，如 \`"A"\`, \`"B"\`, \`"X"\`, \`"Y"\`, \`"LB"\`, \`"RB"\`, \`"LT"\`, \`"RT"\`, \`"Up"\`, \`"Down"\`...`
        } else if (name === 'release') {
          markdownValue = `**release(button)** \\
**release(deviceId, button)** \\
\\
**模拟释放虚拟手柄的指定按键**
- \`deviceId\` (可选): 设备编号或 UUID
- \`button\`: 按键字符串，如 \`"A"\`, \`"B"\`...`
        } else if (name === 'set_thumb') {
          markdownValue = `**set_thumb(axis, value)** \\
**set_thumb(deviceId, axis, value)** \\
\\
**设置模拟摇杆偏转倾斜度**
- \`deviceId\` (可选): 设备编号
- \`axis\`: 轴向名称（左摇杆：\`"LeftX"\`, \`"LeftY"\`；右摇杆：\`"RightX"\`, \`"RightY"\`）
- \`value\`: 浮点数，范围 \`[-1.0, 1.0]\`，\`0.0\` 代表中位。`
        } else if (name === 'set_trigger') {
          markdownValue = `**set_trigger(side, value)** \\
**set_trigger(deviceId, side, value)** \\
\\
**设置模拟手柄左右扳机键深度压力值**
- \`deviceId\` (可选): 设备编号
- \`side\`: 扳机方向，可为 \`"Left"\` 或 \`"Right"\`
- \`value\`: 压力深度浮点数，范围 \`[0.0, 1.0]\`。`
        } else if (name === 'ocr') {
          markdownValue = `**ocr()** \\
**ocr(index)** \\
**ocr(x, y, w, h)** \\
\\
**内置 PaddleOCR / WinRT OCR 本地文本识别**
- \`ocr()\`: 默认识别屏幕第一个标定区域 #1 文本内容
- \`ocr(index)\`: 识别指定的标定区 #index 的文本内容
- \`ocr(x, y, w, h)\`: 精准识别屏幕任意矩形区域的文字

**返回值**：返回识别出来的文本字符串，识别失败时返回空字符串。`
        } else if (name === 'sleep') {
          markdownValue = `**sleep(ms)** \\
\\
**挂起延时休眠**
- \`ms\`: 阻塞式休眠的时长，单位为毫秒（1秒 = 1000毫秒）。`
        } else if (name === 'log') {
          markdownValue = `**log(message)** \\
\\
**在系统运行日志中输出调试信息**
- \`message\`: 日志字符串，会实时输出至下方控制台面板。`
        }

        if (markdownValue) {
          return {
            contents: [
              { value: markdownValue }
            ]
          }
        }
        return null
      }
    })
  }

  // Custom Feishu Light Theme
  monaco.editor.defineTheme('forza-light', {
    base: 'vs',
    inherit: true,
    rules: [
      { token: 'keyword', foreground: '1E54D4', fontStyle: 'bold' },
      { token: 'comment', foreground: '8F959E', fontStyle: 'italic' },
      { token: 'number', foreground: 'FF8800' },
      { token: 'string', foreground: '00B65B' },
      { token: 'identifier', foreground: '1F2329' }
    ],
    colors: {
      'editor.background': '#FFFFFF',
      'editor.foreground': '#1F2329',
      'editorLineNumber.foreground': '#8F959E',
      'editorLineNumber.activeForeground': '#3370FF',
      'editor.lineHighlightBackground': '#F5F6F7',
      'editorCursor.foreground': '#3370FF',
      'editor.selectionBackground': '#E1EDFF',
      'editor.inactiveSelectionBackground': '#F5F6F7'
    }
  })
}

// Resize Observer to handle responsiveness
let resizeObserver: ResizeObserver | null = null

onMounted(async () => {
  setupMonacoRhai()

  // Make sure DOM is fully loaded and structured
  await nextTick()

  if (containerRef.value) {
    editor = monaco.editor.create(containerRef.value, {
      value: props.modelValue,
      language: 'rhai',
      theme: 'forza-light',
      automaticLayout: false, // Will resize manually with ResizeObserver for higher performance
      fontFamily: 'Fira Code, monospace',
      fontSize: 13,
      lineHeight: 20,
      minimap: { enabled: false },
      tabSize: 2,
      cursorBlinking: 'smooth',
      smoothScrolling: true,
      padding: { top: 12, bottom: 12 },
      hover: { enabled: true }
    })

    // Listen to changes to emit update to parent
    editor.onDidChangeModelContent(() => {
      if (editor) {
        const val = editor.getValue()
        if (val !== props.modelValue) {
          isUpdating = true
          emit('update:modelValue', val)
          isUpdating = false
        }
      }
    })

    // Bind Ctrl+S command to save
    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, () => {
      emit('save')
    })

    // Observe size changes of container and layout editor
    resizeObserver = new ResizeObserver(() => {
      editor?.layout()
    })
    resizeObserver.observe(containerRef.value)
  }
})

onUnmounted(() => {
  resizeObserver?.disconnect()
  resizeObserver = null
  editor?.dispose()
  editor = null
  activeLineDecoration = []
})

// Sync value from parent
watch(
  () => props.modelValue,
  (newVal) => {
    if (editor && !isUpdating) {
      const currentVal = editor.getValue()
      if (newVal !== currentVal) {
        editor.setValue(newVal)
      }
    }
  }
)

// Highlight active execution line from parent
watch(
  () => props.activeLine,
  (newVal) => {
    if (editor) {
      const line = newVal || 0
      const decorations: monaco.editor.IModelDeltaDecoration[] = []
      if (line > 0) {
        decorations.push({
          range: new monaco.Range(line, 1, line, 1),
          options: {
            isWholeLine: true,
            className: 'active-execution-line',
            marginClassName: 'active-execution-line-margin'
          }
        })
        
        // Auto scroll to make it visible
        editor.revealLineInCenter(line)
      }
      activeLineDecoration = editor.deltaDecorations(activeLineDecoration, decorations)
    }
  },
  { immediate: true }
)
</script>

<template>
  <div class="relative h-full w-full overflow-hidden rounded-lg border border-border bg-surface">
    <div ref="containerRef" class="absolute inset-0 h-full w-full"></div>
  </div>
</template>

<style>
.active-execution-line {
  background: rgba(34, 197, 94, 0.12) !important;
  border-left: 3px solid oklch(var(--primary)) !important;
}

.active-execution-line-margin {
  background: rgba(34, 197, 94, 0.2) !important;
  font-weight: bold;
}
</style>
