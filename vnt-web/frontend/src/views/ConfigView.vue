<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'

import { deleteConfig, getConfig, saveConfig } from '@/api/vnt'
import SectionCard from '@/components/SectionCard.vue'
import { DEFAULT_CONFIG_TEMPLATE } from '@/constants/defaultConfig'
import { useAppStore } from '@/stores/app'

const app = useAppStore()

const selectedFile = ref('')
const draftFileName = ref('')
const editorContent = ref(DEFAULT_CONFIG_TEMPLATE)
const loadedContent = ref('')
const loadingEditor = ref(false)

const normalizedFileName = computed(() => {
  const value = draftFileName.value.trim()
  if (!value) {
    return ''
  }

  return value.endsWith('.toml') ? value : `${value}.toml`
})

const isDirty = computed(
  () =>
    editorContent.value !== loadedContent.value ||
    normalizedFileName.value !== selectedFile.value,
)

async function loadConfig(fileName: string, force = false) {
  if (!force && isDirty.value && !window.confirm('当前内容未保存，确定切换配置吗？')) {
    return
  }

  loadingEditor.value = true
  try {
    const content = await getConfig(fileName)
    selectedFile.value = fileName
    draftFileName.value = fileName
    editorContent.value = content
    loadedContent.value = content
  } catch (error) {
    app.setNotice('error', (error as Error).message, 4800)
  } finally {
    loadingEditor.value = false
  }
}

function createNewConfig() {
  if (isDirty.value && !window.confirm('当前内容未保存，确定创建新的配置草稿吗？')) {
    return
  }

  const stamp = new Date().toISOString().replaceAll(':', '-').slice(0, 19)
  selectedFile.value = ''
  draftFileName.value = `config-${stamp}.toml`
  editorContent.value = DEFAULT_CONFIG_TEMPLATE
  loadedContent.value = ''
}

async function saveCurrentConfig() {
  const fileName = normalizedFileName.value
  if (!fileName) {
    app.setNotice('error', '请先填写配置文件名')
    return
  }

  loadingEditor.value = true
  try {
    await saveConfig(fileName, editorContent.value)
    await app.fetchConfigList()
    selectedFile.value = fileName
    draftFileName.value = fileName
    loadedContent.value = editorContent.value
    app.setNotice('success', `已保存 ${fileName}`)
  } catch (error) {
    app.setNotice('error', (error as Error).message, 5000)
  } finally {
    loadingEditor.value = false
  }
}

async function removeCurrentConfig() {
  if (!selectedFile.value) {
    app.setNotice('error', '当前还没有已保存的配置')
    return
  }

  if (!window.confirm(`确认删除 ${selectedFile.value} 吗？`)) {
    return
  }

  loadingEditor.value = true
  try {
    await deleteConfig(selectedFile.value)
    await app.fetchConfigList()
    if (app.configList.value.length > 0) {
      await loadConfig(app.configList.value[0].file_name, true)
    } else {
      createNewConfig()
    }
    app.setNotice('success', '配置已删除')
  } catch (error) {
    app.setNotice('error', (error as Error).message, 5000)
  } finally {
    loadingEditor.value = false
  }
}

async function saveIfDirty() {
  if (isDirty.value) {
    await saveCurrentConfig()
  }
}

async function startCurrent() {
  await saveIfDirty()
  if (!normalizedFileName.value) {
    return
  }
  await app.start(normalizedFileName.value)
}

async function restartCurrent() {
  await saveIfDirty()
  if (!normalizedFileName.value) {
    return
  }
  await app.restart(normalizedFileName.value)
}

watch(
  () => app.configList.value,
  async (configs) => {
    if (selectedFile.value || configs.length === 0) {
      return
    }

    const preferred =
      configs.find((item) => item.file_name === app.info.value.current_config_file) ?? configs[0]
    await loadConfig(preferred.file_name, true)
  },
  { immediate: true },
)

onMounted(() => {
  if (app.configList.value.length === 0 && !selectedFile.value) {
    createNewConfig()
  }
})
</script>

<template>
  <div class="config-layout">
    <SectionCard title="配置文件" subtitle="左侧保留配置索引，右侧直接编辑 TOML。">
      <template #action>
        <div class="inline-actions">
          <button class="button ghost" @click="app.fetchConfigList()">刷新</button>
          <button class="button primary" @click="createNewConfig()">新建</button>
        </div>
      </template>

      <div class="config-list">
        <button
          v-for="config in app.configList.value"
          :key="config.file_name"
          class="config-item"
          :class="{ active: config.file_name === selectedFile }"
          @click="loadConfig(config.file_name)"
        >
          <strong>{{ config.config_name }}</strong>
          <span>{{ config.file_name }}</span>
        </button>
        <div v-if="app.configList.value.length === 0" class="empty-state">
          还没有配置文件，点击“新建”先生成一个草稿。
        </div>
      </div>
    </SectionCard>

    <SectionCard title="编辑器" subtitle="这里保持和后端接口一致，不额外改配置格式。">
      <template #action>
        <div class="inline-actions">
          <button class="button ghost" :disabled="app.loading.value" @click="app.stop()">停止</button>
          <button class="button ghost" :disabled="loadingEditor" @click="removeCurrentConfig()">删除</button>
          <button class="button ghost" :disabled="loadingEditor" @click="saveCurrentConfig()">保存</button>
          <button class="button ghost" :disabled="app.loading.value" @click="restartCurrent()">重启</button>
          <button class="button primary" :disabled="app.loading.value" @click="startCurrent()">启动</button>
        </div>
      </template>

      <div class="editor-toolbar">
        <label class="field">
          <span>文件名</span>
          <input v-model="draftFileName" type="text" placeholder="example.toml" />
        </label>
        <div class="toolbar-meta">
          <span class="inline-pill" :class="{ ok: !isDirty }">
            {{ isDirty ? '未保存' : '已同步' }}
          </span>
          <span class="inline-pill" :class="{ ok: app.info.value.current_config_file === normalizedFileName }">
            {{ app.info.value.current_config_file === normalizedFileName ? '当前运行配置' : '未运行' }}
          </span>
        </div>
      </div>

      <textarea
        v-model="editorContent"
        class="editor"
        spellcheck="false"
        placeholder="在这里编辑 TOML 配置"
      />

      <div class="editor-foot">
        <span>{{ loadingEditor ? '处理中...' : `${editorContent.length} chars` }}</span>
        <span>{{ normalizedFileName || '未命名配置' }}</span>
      </div>
    </SectionCard>
  </div>
</template>

<style scoped>
.config-layout {
  display: grid;
  gap: 1rem;
  grid-template-columns: minmax(240px, 320px) minmax(0, 1fr);
}

.inline-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 0.65rem;
}

.config-list {
  display: flex;
  max-height: 38rem;
  flex-direction: column;
  gap: 0.75rem;
  overflow: auto;
}

.config-item {
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 1rem;
  background: rgba(255, 255, 255, 0.02);
  padding: 0.95rem 1rem;
  color: inherit;
  text-align: left;
}

.config-item span {
  color: var(--text-soft);
  font-size: 0.88rem;
}

.config-item.active {
  border-color: rgba(124, 155, 255, 0.3);
  background: rgba(124, 155, 255, 0.08);
}

.editor-toolbar,
.editor-foot {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
}

.field {
  display: flex;
  min-width: 0;
  flex: 1;
  flex-direction: column;
  gap: 0.5rem;
}

.field span {
  color: var(--text-soft);
  font-size: 0.9rem;
}

.toolbar-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 0.6rem;
}

.editor {
  min-height: 34rem;
  resize: vertical;
  border: 1px solid var(--border-strong);
  border-radius: 1.25rem;
  background: rgba(4, 11, 20, 0.92);
  padding: 1rem 1.1rem;
  color: #d7e6fb;
  font-family: 'IBM Plex Mono', 'Cascadia Code', 'Consolas', monospace;
  font-size: 0.95rem;
  line-height: 1.65;
}

.editor-foot {
  color: var(--text-soft);
  font-size: 0.9rem;
}

@media (max-width: 1100px) {
  .config-layout {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 720px) {
  .editor-toolbar,
  .editor-foot {
    align-items: flex-start;
    flex-direction: column;
  }
}
</style>
