'use client'

import { useState, useEffect } from 'react'
import { queryAPI } from '@/lib/api'
import { formatDateTime, downloadFile } from '@/lib/utils'

interface QueryResult {
  data: any[]
  elapsed_ms: number
  row_count: number
}

interface QueryHistory {
  sql: string
  timestamp: string
  success: boolean
  row_count?: number
  elapsed_ms?: number
  error?: string
}

export default function QueryPage() {
  const [sql, setSql] = useState('SELECT * FROM public.users LIMIT 10;')
  const [result, setResult] = useState<QueryResult | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')
  const [history, setHistory] = useState<QueryHistory[]>([])

  // 加载历史记录
  useEffect(() => {
    const saved = localStorage.getItem('query_history')
    if (saved) {
      setHistory(JSON.parse(saved))
    }
  }, [])

  // 监听数据库和 Schema 切换
  useEffect(() => {
    if (typeof window !== 'undefined') {
      const handleChange = () => {
        setResult(null)
        setError('')
      }
      window.addEventListener('database-changed', handleChange)
      window.addEventListener('schema-changed', handleChange)
      return () => {
        window.removeEventListener('database-changed', handleChange)
        window.removeEventListener('schema-changed', handleChange)
      }
    }
  }, [])

  // 保存历史记录
  const saveHistory = (item: QueryHistory) => {
    const newHistory = [item, ...history].slice(0, 50) // 保留最近 50 条
    setHistory(newHistory)
    localStorage.setItem('query_history', JSON.stringify(newHistory))
  }

  const executeQuery = async () => {
    if (!sql.trim()) return

    setLoading(true)
    setError('')
    setResult(null)

    try {
      const response = await queryAPI.execute(sql)
      console.log('SQL 查询响应:', response.data)
      
      // 确保返回的数据格式正确
      const data = response.data
      if (data && Array.isArray(data.data)) {
        setResult(data)
        saveHistory({
          sql,
          timestamp: new Date().toISOString(),
          success: true,
          row_count: data.row_count,
          elapsed_ms: data.elapsed_ms,
        })
      } else {
        console.error('API 返回格式错误:', data)
        setError('返回的数据格式不正确')
      }
    } catch (err: any) {
      console.error('SQL 查询失败:', err)
      const errorMsg = err.response?.data?.error || err.message || '查询失败'
      setError(errorMsg)
      saveHistory({
        sql,
        timestamp: new Date().toISOString(),
        success: false,
        error: errorMsg,
      })
    } finally {
      setLoading(false)
    }
  }

  const exportCSV = async () => {
    if (!sql.trim()) return

    try {
      const response = await queryAPI.exportCSV(sql)
      downloadFile(response.data, `query_${Date.now()}.csv`)
    } catch (err: any) {
      alert('导出失败：' + (err.response?.data?.error || err.message))
    }
  }

  const clearHistory = () => {
    setHistory([])
    localStorage.removeItem('query_history')
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-semibold text-gray-800">SQL 查询器</h1>
      </div>

      <div className="grid grid-cols-12 gap-6">
        {/* 查询编辑器 */}
        <div className="col-span-8 space-y-4">
          <div className="card p-4">
            <div className="mb-3">
              <label className="block text-sm font-medium text-gray-700 mb-2">
                SQL 查询
              </label>
              <textarea
                value={sql}
                onChange={(e) => setSql(e.target.value)}
                className="w-full h-48 input-base font-mono text-sm resize-none"
                placeholder="输入 SQL 查询语句..."
              />
            </div>

            <div className="flex items-center space-x-3">
              <button
                onClick={executeQuery}
                disabled={loading}
                className="btn-success"
              >
                <i className="fas fa-play text-xs mr-2"></i>
                {loading ? '执行中...' : '执行查询'}
              </button>

              <button onClick={exportCSV} className="btn-default">
                <i className="fas fa-download text-xs mr-2"></i>
                导出 CSV
              </button>

              <button
                onClick={() => setSql('')}
                className="btn-default"
              >
                <i className="fas fa-eraser text-xs mr-2"></i>
                清空
              </button>
            </div>
          </div>

          {/* 查询结果 */}
          {error && (
            <div className="card p-4 bg-red-50 border-red-200">
              <div className="flex items-start space-x-3">
                <i className="fas fa-exclamation-circle text-red-500 mt-0.5"></i>
                <div className="flex-1">
                  <p className="text-sm font-medium text-red-800">查询错误</p>
                  <p className="text-sm text-red-600 mt-1">{error}</p>
                </div>
              </div>
            </div>
          )}

          {result && (
            <div className="card">
              <div className="px-4 py-3 border-b border-gray-100 flex items-center justify-between">
                <h3 className="text-sm font-semibold text-gray-700">查询结果</h3>
                <div className="flex items-center space-x-4 text-xs text-gray-500">
                  <span>
                    <i className="fas fa-clock mr-1"></i>
                    {result.elapsed_ms} ms
                  </span>
                  <span>
                    <i className="fas fa-list mr-1"></i>
                    {result.row_count} 行
                  </span>
                </div>
              </div>
              <div className="overflow-auto max-h-[600px]">
                {result.data.length === 0 ? (
                  <div className="p-8 text-center">
                    <i className="fas fa-inbox text-3xl text-gray-300 mb-3"></i>
                    <p className="text-sm text-gray-500">查询结果为空</p>
                  </div>
                ) : (
                  <table className="w-full enterprise-table">
                    <thead>
                      <tr>
                        {Object.keys(result.data[0]).map((key) => (
                          <th key={key}>{key}</th>
                        ))}
                      </tr>
                    </thead>
                    <tbody>
                      {result.data.map((row, idx) => (
                        <tr key={idx}>
                          {Object.values(row).map((value: any, i) => (
                            <td key={i}>
                              {value === null ? (
                                <span className="text-gray-400 italic">NULL</span>
                              ) : typeof value === 'object' ? (
                                <code className="text-xs">{JSON.stringify(value)}</code>
                              ) : (
                                String(value)
                              )}
                            </td>
                          ))}
                        </tr>
                      ))}
                    </tbody>
                  </table>
                )}
              </div>
            </div>
          )}
        </div>

        {/* 查询历史 */}
        <div className="col-span-4">
          <div className="card">
            <div className="px-4 py-3 border-b border-gray-100 flex items-center justify-between">
              <h3 className="text-sm font-semibold text-gray-700">查询历史</h3>
              {history.length > 0 && (
                <button
                  onClick={clearHistory}
                  className="text-xs text-red-600 hover:text-red-700"
                >
                  <i className="fas fa-trash mr-1"></i>
                  清空
                </button>
              )}
            </div>
            <div className="divide-y divide-gray-100 max-h-[700px] overflow-y-auto">
              {history.length === 0 ? (
                <div className="p-8 text-center">
                  <i className="fas fa-history text-3xl text-gray-300 mb-3"></i>
                  <p className="text-sm text-gray-500">暂无查询历史</p>
                </div>
              ) : (
                history.map((item, idx) => (
                  <div
                    key={idx}
                    className="p-3 hover:bg-gray-50 cursor-pointer transition-colors"
                    onClick={() => setSql(item.sql)}
                  >
                    <div className="flex items-start space-x-2">
                      <i
                        className={`fas ${
                          item.success ? 'fa-check-circle text-green-500' : 'fa-times-circle text-red-500'
                        } text-sm mt-0.5`}
                      ></i>
                      <div className="flex-1 min-w-0">
                        <p className="text-xs font-mono text-gray-800 line-clamp-2">
                          {item.sql}
                        </p>
                        <div className="flex items-center space-x-2 mt-1 text-xs text-gray-500">
                          <span>{formatDateTime(item.timestamp)}</span>
                          {item.success && (
                            <>
                              <span>•</span>
                              <span>{item.row_count} 行</span>
                              <span>•</span>
                              <span>{item.elapsed_ms} ms</span>
                            </>
                          )}
                        </div>
                        {item.error && (
                          <p className="text-xs text-red-600 mt-1">{item.error}</p>
                        )}
                      </div>
                    </div>
                  </div>
                ))
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

