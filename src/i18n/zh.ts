export default {
  app: {
    title: '文件扫描器',
    pleaseSelect: '请选择文件或目录进行分析',
    menu: {
      file: '文件',
      view: '视图',
      help: '帮助'
    }
  },
  sizes: {
    byte: 'B',
    kilobyte: 'KB',
    megabyte: 'MB',
    gigabyte: 'GB',
    terabyte: 'TB'
  },
  fileStatistics: {
    title: '文件统计',
    loading: '正在加载统计数据...',
    noData: '暂无数据',
    files: '文件',
    folders: '文件夹',
    totalSize: '总大小',
    fileTypes: {
      image: '图片',
      video: '视频',
      audio: '音频',
      document: '文档',
      archive: '压缩包',
      code: '代码',
      executable: '可执行文件',
      other: '其他'
    }
  },
  selectionSummary: {
    noSelection: '未选择任何文件',
    selected: '{count} 个项目已选择',
    totalSize: '总大小',
    typeDistribution: '类型分布',
    selectedFiles: '已选择的文件',
    actions: {
      delete: '删除',
      clear: '清除选择'
    }
  },
  dialog: {
    confirm: '确认',
    cancel: '取消',
    delete: {
      title: '确认删除',
      message: '确定要删除选中的文件吗？此操作不可撤销。'
    },
    error: {
      title: '错误',
      unknown: '发生未知错误'
    }
  }
}