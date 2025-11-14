'use client'

import { useState } from 'react'
import { useAppStore } from '@/lib/store'

interface DatabaseForm {
  name: string
  host: string
  port: number
  database: string
  username: string
  password: string
  description: string
}

import ConnectionWarning from '@/components/ConnectionWarning'

export default function ConnectionsPage() {
  const { databases, addDatabase, removeDatabase, setCurrentDatabase } = useAppStore()
  const [showForm, setShowForm] = useState(false)
  const [formData, setFormData] = useState<DatabaseForm>({
    name: '',
    host: 'localhost',
    port: 5432,
    database: '',
    username: 'postgres',
    password: '',
    description: '',
  })

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    const newDb = {
      id: Date.now().toString(),
      name: formData.name,
      host: formData.host,
      port: formData.port,
      database: formData.database,
      description: formData.description,
    }
    addDatabase(newDb)
    setShowForm(false)
    setFormData({
      name: '',
      host: 'localhost',
      port: 5432,
      database: '',
      username: 'postgres',
      password: '',
      description: '',
    })
  }

  const handleDelete = (id: string) => {
    if (confirm('确定删除此连接吗？')) {
      removeDatabase(id)
    }
  }

  const handleTest = async () => {
    alert('测试连接功能待实现')
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-semibold text-gray-900">数据库连接</h1>
          <p className="text-sm text-gray-500 mt-1">管理您的数据库连接配置</p>
        </div>
        <button
          onClick={() => setShowForm(true)}
          className="btn-primary"
        >
          <i className="fas fa-plus text-xs mr-2"></i>
          添加连接
        </button>
      </div>

      {/* 警告提示 */}
      <ConnectionWarning />

      {/* 当前实际连接 */}
      <div className="card p-4 bg-blue-50 border-blue-200">
        <div className="flex items-start space-x-3">
          <i className="fas fa-info-circle text-blue-500 mt-0.5"></i>
          <div className="flex-1">
            <h3 className="text-sm font-semibold text-blue-900 mb-1">当前实际连接</h3>
            <p className="text-sm text-blue-700">
              后端连接：<code className="bg-blue-100 px-2 py-0.5 rounded font-mono">localhost:5432/crestrail</code>
            </p>
            <p className="text-xs text-blue-600 mt-2">
              由后端 .env 文件的 DATABASE_URL 配置
            </p>
          </div>
        </div>
      </div>

      {/* 连接列表 */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {databases.map((db) => (
          <div key={db.id} className="card p-5">
            <div className="flex items-start justify-between mb-4">
              <div className="flex items-start space-x-3">
                <div className="w-10 h-10 bg-blue-100 rounded-lg flex items-center justify-center flex-shrink-0">
                  <i className="fas fa-database text-blue-600"></i>
                </div>
                <div className="flex-1 min-w-0">
                  <h3 className="text-sm font-semibold text-gray-900 truncate">
                    {db.name}
                  </h3>
                  <p className="text-xs text-gray-500 mt-0.5 truncate">
                    {db.host}:{db.port}
                  </p>
                </div>
              </div>
              <button
                onClick={() => handleDelete(db.id)}
                className="text-gray-400 hover:text-red-600 transition-colors"
                disabled={db.id === 'default'}
              >
                <i className="fas fa-trash text-xs"></i>
              </button>
            </div>

            <div className="space-y-2 mb-4">
              <div className="flex items-center justify-between text-xs">
                <span className="text-gray-500">数据库</span>
                <span className="font-mono text-gray-900">{db.database}</span>
              </div>
              {db.description && (
                <p className="text-xs text-gray-500">{db.description}</p>
              )}
            </div>

            <div className="flex items-center space-x-2">
              <button
                onClick={() => setCurrentDatabase(db)}
                className="flex-1 btn-primary text-xs py-1.5"
              >
                使用此连接
              </button>
              <button className="btn-default text-xs py-1.5 px-3">
                <i className="fas fa-edit"></i>
              </button>
            </div>
          </div>
        ))}
      </div>

      {/* 添加连接表单 */}
      {showForm && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
          <div className="bg-white rounded-lg shadow-xl max-w-2xl w-full max-h-[90vh] overflow-y-auto">
            <div className="px-6 py-4 border-b border-gray-200 flex items-center justify-between">
              <h2 className="text-lg font-semibold text-gray-900">添加数据库连接</h2>
              <button
                onClick={() => setShowForm(false)}
                className="text-gray-400 hover:text-gray-600"
              >
                <i className="fas fa-times"></i>
              </button>
            </div>

            <form onSubmit={handleSubmit} className="p-6 space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  连接名称 *
                </label>
                <input
                  type="text"
                  required
                  value={formData.name}
                  onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                  className="input-base w-full"
                  placeholder="例如：生产环境数据库"
                />
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    主机地址 *
                  </label>
                  <input
                    type="text"
                    required
                    value={formData.host}
                    onChange={(e) => setFormData({ ...formData, host: e.target.value })}
                    className="input-base w-full"
                    placeholder="localhost"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    端口 *
                  </label>
                  <input
                    type="number"
                    required
                    value={formData.port}
                    onChange={(e) => setFormData({ ...formData, port: parseInt(e.target.value) })}
                    className="input-base w-full"
                    placeholder="5432"
                  />
                </div>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  数据库名 *
                </label>
                <input
                  type="text"
                  required
                  value={formData.database}
                  onChange={(e) => setFormData({ ...formData, database: e.target.value })}
                  className="input-base w-full"
                  placeholder="mydb"
                />
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    用户名 *
                  </label>
                  <input
                    type="text"
                    required
                    value={formData.username}
                    onChange={(e) => setFormData({ ...formData, username: e.target.value })}
                    className="input-base w-full"
                    placeholder="postgres"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    密码 *
                  </label>
                  <input
                    type="password"
                    required
                    value={formData.password}
                    onChange={(e) => setFormData({ ...formData, password: e.target.value })}
                    className="input-base w-full"
                    placeholder="••••••"
                  />
                </div>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  描述
                </label>
                <textarea
                  value={formData.description}
                  onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                  className="input-base w-full"
                  rows={3}
                  placeholder="可选的连接描述"
                />
              </div>

              <div className="flex items-center justify-end space-x-3 pt-4">
                <button
                  type="button"
                  onClick={handleTest}
                  className="btn-default"
                >
                  测试连接
                </button>
                <button
                  type="button"
                  onClick={() => setShowForm(false)}
                  className="btn-default"
                >
                  取消
                </button>
                <button type="submit" className="btn-primary">
                  保存连接
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  )
}

