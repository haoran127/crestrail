'use client'

import { useState, useEffect } from 'react'
import { useAppStore } from '@/lib/store'
import { schemaAPI } from '@/lib/api'

export default function SchemaSelector() {
  const { currentSchema, setCurrentSchema } = useAppStore()
  const [schemas, setSchemas] = useState<any[]>([])
  const [showMenu, setShowMenu] = useState(false)
  const [loading, setLoading] = useState(false)
  const [mounted, setMounted] = useState(false)

  useEffect(() => {
    setMounted(true)
    loadSchemas()
  }, [])

  // 监听数据库连接切换
  useEffect(() => {
    if (typeof window !== 'undefined') {
      const handleConnectionChange = () => {
        loadSchemas()
      }
      // 监听新的 connection-changed 事件
      window.addEventListener('connection-changed', handleConnectionChange)
      return () => window.removeEventListener('connection-changed', handleConnectionChange)
    }
  }, [])

  const loadSchemas = async () => {
    setLoading(true)
    try {
      const response = await schemaAPI.listSchemas()
      const data = Array.isArray(response.data) ? response.data : []
      setSchemas(data)
      // 如果当前 schema 不在列表中，选择第一个
      if (data.length > 0 && !data.find((s: any) => s.schema_name === currentSchema)) {
        setCurrentSchema(data[0].schema_name)
      }
    } catch (err) {
      console.error('加载 schemas 失败:', err)
      setSchemas([])
    } finally {
      setLoading(false)
    }
  }

  const handleSchemaChange = (schemaName: string) => {
    setCurrentSchema(schemaName)
    setShowMenu(false)
    // 触发自定义事件通知页面刷新
    if (typeof window !== 'undefined') {
      window.dispatchEvent(new Event('schema-changed'))
    }
  }

  if (!mounted) {
    return (
      <div className="px-3 py-2 bg-gray-50 rounded-lg border border-gray-200">
        <div className="flex items-center space-x-2">
          <i className="fas fa-layer-group text-gray-400 text-xs"></i>
          <span className="text-xs text-gray-700">加载中...</span>
        </div>
      </div>
    )
  }

  return (
    <div className="relative">
      <button
        onClick={() => setShowMenu(!showMenu)}
        className="w-full px-3 py-2 bg-gray-50 rounded-lg border border-gray-200 hover:border-gray-300 transition-colors"
      >
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-2 min-w-0 flex-1">
            <i className="fas fa-layer-group text-gray-400 text-xs flex-shrink-0"></i>
            <div className="flex-1 min-w-0 text-left">
              <p className="text-[10px] text-gray-500 uppercase">Schema</p>
              <p className="text-xs font-medium text-gray-700 truncate">
                {currentSchema || '选择 Schema'}
              </p>
            </div>
          </div>
          <i className={`fas fa-chevron-down text-gray-400 text-[10px] flex-shrink-0 transition-transform ${showMenu ? 'rotate-180' : ''}`}></i>
        </div>
      </button>

      {/* 下拉菜单 */}
      {showMenu && (
        <>
          {/* 遮罩层 */}
          <div
            className="fixed inset-0 z-40"
            onClick={() => setShowMenu(false)}
          ></div>

          {/* 菜单内容 */}
          <div className="absolute top-full left-0 right-0 mt-1 bg-white border border-gray-200 rounded-lg shadow-lg z-50 max-h-60 overflow-y-auto">
            <div className="p-2">
              {loading ? (
                <div className="px-3 py-4 text-center">
                  <i className="fas fa-spinner fa-spin text-gray-400"></i>
                  <p className="text-xs text-gray-500 mt-1">加载中...</p>
                </div>
              ) : schemas.length === 0 ? (
                <div className="px-3 py-4 text-center">
                  <p className="text-xs text-gray-500">暂无 Schema</p>
                </div>
              ) : (
                <div className="space-y-0.5">
                  {schemas.map((schema: any) => (
                    <button
                      key={schema.schema_name}
                      onClick={() => handleSchemaChange(schema.schema_name)}
                      className={`w-full px-3 py-2 rounded-md text-left transition-colors ${
                        currentSchema === schema.schema_name
                          ? 'bg-blue-50 text-blue-600'
                          : 'hover:bg-gray-50 text-gray-700'
                      }`}
                    >
                      <div className="flex items-center justify-between">
                        <div className="flex items-center space-x-2 flex-1 min-w-0">
                          <i
                            className={`fas fa-layer-group text-xs ${
                              currentSchema === schema.schema_name
                                ? 'text-blue-600'
                                : 'text-gray-400'
                            }`}
                          ></i>
                          <div className="flex-1 min-w-0">
                            <p className="text-xs font-medium truncate">
                              {schema.schema_name}
                            </p>
                            <p className="text-[10px] text-gray-500">
                              {schema.table_count} 张表
                            </p>
                          </div>
                        </div>
                        {currentSchema === schema.schema_name && (
                          <i className="fas fa-check text-blue-600 text-xs"></i>
                        )}
                      </div>
                    </button>
                  ))}
                </div>
              )}
            </div>
          </div>
        </>
      )}
    </div>
  )
}

