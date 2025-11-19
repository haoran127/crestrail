'use client'

import { useState, useEffect } from 'react'
import { adminAPI } from '@/lib/api'

interface Tenant {
  tenant_id: number
  tenant_name: string
  slug: string
  status: string
  contact_email: string | null
  database_count: number
  schema_count: number
  user_count: number
  users: string[]
  created_at: string
}

export default function AdminTenantsPage() {
  const [tenants, setTenants] = useState<Tenant[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState('')
  const [showCreateModal, setShowCreateModal] = useState(false)
  
  // 创建租户表单
  const [newTenant, setNewTenant] = useState({
    name: '',
    slug: '',
    contact_email: '',
  })

  useEffect(() => {
    loadTenants()
  }, [])

  const loadTenants = async () => {
    setLoading(true)
    setError('')
    try {
      const response = await adminAPI.listAllTenants()
      setTenants(response.data || [])
    } catch (err: any) {
      setError(err.response?.data?.error || '加载租户列表失败')
      console.error('加载租户失败:', err)
    } finally {
      setLoading(false)
    }
  }

  const handleCreateTenant = async (e: React.FormEvent) => {
    e.preventDefault()
    
    try {
      await adminAPI.createTenant(newTenant)
      setShowCreateModal(false)
      setNewTenant({ name: '', slug: '', contact_email: '' })
      loadTenants()
      alert('租户创建成功！')
    } catch (err: any) {
      alert(err.response?.data?.error || '创建租户失败')
      console.error('创建租户失败:', err)
    }
  }

  return (
    <div className="space-y-6">
      {/* 标题栏 */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-semibold text-gray-900">租户管理</h1>
          <p className="text-sm text-gray-500 mt-1">管理系统中的所有租户</p>
        </div>
        <button
          onClick={() => setShowCreateModal(true)}
          className="btn-primary text-sm"
        >
          <i className="fas fa-plus mr-2"></i>
          创建租户
        </button>
      </div>

      {/* 错误提示 */}
      {error && (
        <div className="card p-4 bg-red-50 border-red-200">
          <p className="text-sm text-red-600">{error}</p>
        </div>
      )}

      {/* 租户列表 */}
      {loading ? (
        <div className="card p-12 text-center">
          <i className="fas fa-spinner fa-spin text-3xl text-gray-400 mb-3"></i>
          <p className="text-sm text-gray-500">加载中...</p>
        </div>
      ) : tenants.length === 0 ? (
        <div className="card p-12 text-center">
          <i className="fas fa-building text-5xl text-gray-300 mb-4"></i>
          <p className="text-gray-500 mb-2">暂无租户</p>
          <button
            onClick={() => setShowCreateModal(true)}
            className="btn-primary text-sm mt-4"
          >
            创建第一个租户
          </button>
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {tenants.map((tenant) => (
            <div key={tenant.tenant_id} className="card p-6 hover:shadow-lg transition-shadow">
              {/* 租户头部 */}
              <div className="flex items-start justify-between mb-4">
                <div className="flex items-center space-x-3">
                  <div className="w-12 h-12 bg-gradient-to-br from-blue-500 to-blue-600 rounded-lg flex items-center justify-center">
                    <i className="fas fa-building text-white text-xl"></i>
                  </div>
                  <div>
                    <h3 className="font-semibold text-gray-900">{tenant.tenant_name}</h3>
                    <p className="text-xs text-gray-500">{tenant.slug}</p>
                  </div>
                </div>
                <span className={`px-2 py-1 text-xs rounded-full ${
                  tenant.status === 'active' 
                    ? 'bg-green-100 text-green-700' 
                    : 'bg-gray-100 text-gray-700'
                }`}>
                  {tenant.status === 'active' ? '活跃' : tenant.status}
                </span>
              </div>

              {/* 统计信息 */}
              <div className="grid grid-cols-3 gap-4 mb-4">
                <div className="text-center">
                  <div className="text-2xl font-bold text-blue-600">{tenant.database_count}</div>
                  <div className="text-xs text-gray-500">数据库</div>
                </div>
                <div className="text-center">
                  <div className="text-2xl font-bold text-green-600">{tenant.schema_count}</div>
                  <div className="text-xs text-gray-500">Schema</div>
                </div>
                <div className="text-center">
                  <div className="text-2xl font-bold text-purple-600">{tenant.user_count}</div>
                  <div className="text-xs text-gray-500">用户</div>
                </div>
              </div>

              {/* 用户列表 */}
              {tenant.users && tenant.users.length > 0 && (
                <div className="border-t border-gray-100 pt-4">
                  <p className="text-xs text-gray-500 mb-2">用户：</p>
                  <div className="flex flex-wrap gap-2">
                    {tenant.users.slice(0, 3).map((user, idx) => (
                      <span key={idx} className="px-2 py-1 bg-gray-100 text-xs rounded">
                        {user}
                      </span>
                    ))}
                    {tenant.users.length > 3 && (
                      <span className="px-2 py-1 bg-gray-100 text-xs rounded text-gray-500">
                        +{tenant.users.length - 3}
                      </span>
                    )}
                  </div>
                </div>
              )}

              {/* 联系方式 */}
              {tenant.contact_email && (
                <div className="border-t border-gray-100 pt-4 mt-4">
                  <p className="text-xs text-gray-500">
                    <i className="fas fa-envelope mr-2"></i>
                    {tenant.contact_email}
                  </p>
                </div>
              )}

              {/* 创建时间 */}
              <div className="text-xs text-gray-400 mt-3">
                创建于 {new Date(tenant.created_at).toLocaleDateString()}
              </div>
            </div>
          ))}
        </div>
      )}

      {/* 创建租户模态框 */}
      {showCreateModal && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg shadow-xl max-w-md w-full mx-4">
            <div className="p-6">
              <h2 className="text-xl font-semibold mb-4">创建新租户</h2>
              
              <form onSubmit={handleCreateTenant} className="space-y-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    租户名称 *
                  </label>
                  <input
                    type="text"
                    required
                    value={newTenant.name}
                    onChange={(e) => setNewTenant({ ...newTenant, name: e.target.value })}
                    className="input-default"
                    placeholder="例如：示例公司"
                  />
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    租户标识 (slug) *
                  </label>
                  <input
                    type="text"
                    required
                    value={newTenant.slug}
                    onChange={(e) => setNewTenant({ ...newTenant, slug: e.target.value.toLowerCase() })}
                    className="input-default"
                    placeholder="例如：example-company"
                  />
                  <p className="text-xs text-gray-500 mt-1">
                    只能包含小写字母、数字和连字符
                  </p>
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    联系邮箱
                  </label>
                  <input
                    type="email"
                    value={newTenant.contact_email}
                    onChange={(e) => setNewTenant({ ...newTenant, contact_email: e.target.value })}
                    className="input-default"
                    placeholder="admin@example.com"
                  />
                </div>

                <div className="flex space-x-3 pt-4">
                  <button
                    type="button"
                    onClick={() => setShowCreateModal(false)}
                    className="btn-default flex-1"
                  >
                    取消
                  </button>
                  <button
                    type="submit"
                    className="btn-primary flex-1"
                  >
                    创建
                  </button>
                </div>
              </form>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
