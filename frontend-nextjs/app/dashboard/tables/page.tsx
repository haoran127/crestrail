'use client'

import { useState, useEffect } from 'react'
import { tableAPI, schemaAPI } from '@/lib/api'
import { downloadFile } from '@/lib/utils'
import { useAppStore } from '@/lib/store'

interface SchemaInfo {
  schema_name: string
  table_count: number
}

export default function TablesPage() {
  const { currentSchema } = useAppStore()
  const [selectedTable, setSelectedTable] = useState('')
  const [records, setRecords] = useState<any[]>([])
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')

  // 监听数据库和 Schema 切换
  useEffect(() => {
    if (typeof window !== 'undefined') {
      const handleChange = () => {
        setSelectedTable('')
        setRecords([])
      }
      window.addEventListener('database-changed', handleChange)
      window.addEventListener('schema-changed', handleChange)
      return () => {
        window.removeEventListener('database-changed', handleChange)
        window.removeEventListener('schema-changed', handleChange)
      }
    }
  }, [])

  const loadRecords = async () => {
    // 验证参数
    if (!currentSchema || !selectedTable) {
      setError('请先输入表名')
      return
    }

    setLoading(true)
    setError('')
    try {
      const response = await tableAPI.getRecords(currentSchema, selectedTable, {
        limit: 100,
      })
      console.log('表数据响应:', response.data)
      setRecords(response.data.data || [])
    } catch (err: any) {
      console.error('加载数据失败:', err)
      setError(err.response?.data?.error || err.message || '加载失败')
      setRecords([])
    } finally {
      setLoading(false)
    }
  }

  const handleExportCSV = async () => {
    if (!currentSchema || !selectedTable) {
      alert('请先输入表名')
      return
    }

    try {
      const response = await tableAPI.exportCSV(currentSchema, selectedTable, {
        limit: 1000,
      })
      downloadFile(response.data, `${currentSchema}_${selectedTable}.csv`)
    } catch (err: any) {
      alert('导出失败：' + (err.response?.data?.error || err.message))
    }
  }

  const handleExportJSON = async () => {
    if (!currentSchema || !selectedTable) {
      alert('请先输入表名')
      return
    }

    try {
      const response = await tableAPI.exportJSON(currentSchema, selectedTable, {
        limit: 1000,
      })
      const blob = new Blob([JSON.stringify(response.data, null, 2)], {
        type: 'application/json',
      })
      downloadFile(blob, `${currentSchema}_${selectedTable}.json`)
    } catch (err: any) {
      alert('导出失败：' + (err.response?.data?.error || err.message))
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-semibold text-gray-800">数据表管理</h1>
          <p className="text-sm text-gray-500 mt-1">
            当前 Schema: <span className="font-mono font-medium text-gray-900">{currentSchema}</span>
          </p>
        </div>
        <div className="flex items-center space-x-3">
          <input
            type="text"
            value={selectedTable}
            onChange={(e) => setSelectedTable(e.target.value)}
            placeholder="输入表名..."
            className="input-base w-64"
          />
          <button 
            onClick={loadRecords} 
            disabled={!currentSchema || !selectedTable || loading}
            className="btn-primary disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <i className={`fas ${loading ? 'fa-spinner fa-spin' : 'fa-search'} text-xs mr-2`}></i>
            {loading ? '查询中...' : '查询'}
          </button>
        </div>
      </div>

      {error && (
        <div className="card p-4 bg-red-50 border-red-200">
          <div className="flex items-start space-x-3">
            <i className="fas fa-exclamation-circle text-red-500 mt-0.5"></i>
            <div className="flex-1">
              <p className="text-sm font-medium text-red-800">错误</p>
              <p className="text-sm text-red-600 mt-1">{error}</p>
            </div>
          </div>
        </div>
      )}

      {selectedTable && (
        <div className="card">
          <div className="px-4 py-3 border-b border-gray-100 flex items-center justify-between">
            <h3 className="text-sm font-semibold text-gray-700">
              {currentSchema}.{selectedTable}
            </h3>
            <div className="flex items-center space-x-2">
              <button 
                onClick={handleExportCSV} 
                disabled={!records.length}
                className="btn-default text-xs disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <i className="fas fa-file-csv text-xs mr-1.5"></i>
                导出 CSV
              </button>
              <button 
                onClick={handleExportJSON} 
                disabled={!records.length}
                className="btn-default text-xs disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <i className="fas fa-file-code text-xs mr-1.5"></i>
                导出 JSON
              </button>
              <button 
                onClick={loadRecords} 
                disabled={loading}
                className="btn-default text-xs disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <i className={`fas ${loading ? 'fa-spinner fa-spin' : 'fa-sync-alt'} text-xs mr-1.5`}></i>
                刷新
              </button>
            </div>
          </div>

          <div className="overflow-auto max-h-[700px]">
            {loading ? (
              <div className="p-12 text-center">
                <i className="fas fa-spinner fa-spin text-2xl text-primary-500"></i>
                <p className="text-sm text-gray-500 mt-2">加载中...</p>
              </div>
            ) : records.length === 0 ? (
              <div className="p-12 text-center">
                <i className="fas fa-inbox text-4xl text-gray-300 mb-4"></i>
                <p className="text-gray-500">暂无数据</p>
              </div>
            ) : (
              <table className="w-full enterprise-table">
                <thead>
                  <tr>
                    {Object.keys(records[0]).map((key) => (
                      <th key={key}>{key}</th>
                    ))}
                  </tr>
                </thead>
                <tbody>
                  {records.map((row, idx) => (
                    <tr key={idx}>
                      {Object.values(row).map((value: any, i) => (
                        <td key={i}>
                          {value === null ? (
                            <span className="text-gray-400 italic">NULL</span>
                          ) : typeof value === 'object' ? (
                            <code className="text-xs bg-gray-100 px-1 py-0.5 rounded">
                              {JSON.stringify(value)}
                            </code>
                          ) : typeof value === 'boolean' ? (
                            <span
                              className={`inline-flex items-center px-2 py-0.5 rounded text-xs font-medium ${
                                value
                                  ? 'bg-green-100 text-green-700'
                                  : 'bg-gray-100 text-gray-700'
                              }`}
                            >
                              {value ? 'true' : 'false'}
                            </span>
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
  )
}

