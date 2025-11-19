'use client'

import { useState, useEffect } from 'react'
import { useRouter } from 'next/navigation'
import { useAppStore } from '@/lib/store'
import { tenantAPI } from '@/lib/api'

export default function TenantConnectionSelector() {
  const router = useRouter()
  const { currentConnection, setCurrentConnection, userConnections, setUserConnections } = useAppStore()
  const [showMenu, setShowMenu] = useState(false)
  const [loading, setLoading] = useState(false)
  const [mounted, setMounted] = useState(false)

  useEffect(() => {
    setMounted(true)
    loadConnections()
  }, [])

  const loadConnections = async () => {
    setLoading(true)
    try {
      const response = await tenantAPI.getMyConnections()
      const connections = Array.isArray(response.data) ? response.data : []
      setUserConnections(connections)
      
      // 如果没有当前连接，选择第一个主连接或第一个连接
      if (!currentConnection && connections.length > 0) {
        const primaryConn = connections.find((c: any) => c.is_primary) || connections[0]
        setCurrentConnection(primaryConn)
        // 切换到该连接
        await switchToConnection(primaryConn.database_id)
      }
    } catch (err: any) {
      // 静默处理错误 - 多租户功能是可选的
      if (err.response?.status === 500) {
        console.log('💡 提示：多租户功能未配置（这不影响基本功能）')
      } else {
        console.error('加载连接失败:', err)
      }
      setUserConnections([])
    } finally {
      setLoading(false)
    }
  }

  const switchToConnection = async (databaseId: number) => {
    try {
      await tenantAPI.switchConnection(databaseId)
      console.log(`已切换到连接 ${databaseId}`)
    } catch (err) {
      console.error('切换连接失败:', err)
    }
  }

  const handleConnectionChange = async (conn: any) => {
    setCurrentConnection(conn)
    setShowMenu(false)
    
    // 调用后端 API 切换连接
    await switchToConnection(conn.database_id)
    
    // 触发自定义事件通知页面刷新
    if (typeof window !== 'undefined') {
      window.dispatchEvent(new Event('connection-changed'))
    }
  }

  if (!mounted) {
    return (
      <div className="px-3 py-2 bg-gray-50 rounded-lg border border-gray-200">
        <div className="flex items-center space-x-2">
          <i className="fas fa-database text-gray-400 text-xs"></i>
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
            <i className="fas fa-database text-gray-400 text-xs flex-shrink-0"></i>
            <div className="flex-1 min-w-0 text-left">
              <p className="text-[10px] text-gray-500 uppercase">租户 / 数据库</p>
              <p className="text-xs font-medium text-gray-700 truncate">
                {currentConnection ? (
                  <>
                    {currentConnection.tenant_name} / {currentConnection.connection_name}
                  </>
                ) : (
                  '选择连接'
                )}
              </p>
              {currentConnection && (
                <p className="text-[10px] text-gray-500 truncate">
                  {currentConnection.db_host}:{currentConnection.db_port}/{currentConnection.db_name}
                </p>
              )}
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
          <div className="absolute top-full left-0 right-0 mt-1 bg-white border border-gray-200 rounded-lg shadow-lg z-50 max-h-80 overflow-y-auto">
            <div className="p-2">
              {loading ? (
                <div className="px-3 py-4 text-center">
                  <i className="fas fa-spinner fa-spin text-gray-400"></i>
                  <p className="text-xs text-gray-500 mt-1">加载中...</p>
                </div>
              ) : userConnections.length === 0 ? (
                <div className="px-3 py-4 text-center">
                  <p className="text-xs text-gray-500">暂无可用连接</p>
                  <button
                    onClick={() => {
                      setShowMenu(false)
                      router.push('/dashboard/connections')
                    }}
                    className="mt-2 text-xs text-blue-600 hover:text-blue-700"
                  >
                    + 添加连接
                  </button>
                </div>
              ) : (
                <>
                  {/* 按租户分组显示 */}
                  {Object.entries(
                    userConnections.reduce((acc: any, conn: any) => {
                      if (!acc[conn.tenant_name]) {
                        acc[conn.tenant_name] = []
                      }
                      acc[conn.tenant_name].push(conn)
                      return acc
                    }, {})
                  ).map(([tenantName, connections]: [string, any]) => (
                    <div key={tenantName} className="mb-2 last:mb-0">
                      <div className="px-2 py-1 text-[10px] font-semibold text-gray-500 uppercase">
                        {tenantName}
                      </div>
                      <div className="space-y-0.5">
                        {connections.map((conn: any) => (
                          <button
                            key={conn.database_id}
                            onClick={() => handleConnectionChange(conn)}
                            className={`w-full px-3 py-2 rounded-md text-left transition-colors ${
                              currentConnection?.database_id === conn.database_id
                                ? 'bg-blue-50 text-blue-600'
                                : 'hover:bg-gray-50 text-gray-700'
                            }`}
                          >
                            <div className="flex items-center justify-between">
                              <div className="flex items-center space-x-2 flex-1 min-w-0">
                                {conn.is_primary && (
                                  <i className="fas fa-star text-yellow-500 text-xs flex-shrink-0" title="主连接"></i>
                                )}
                                <div className="flex-1 min-w-0">
                                  <p className="text-xs font-medium truncate">
                                    {conn.connection_name}
                                  </p>
                                  <p className="text-[10px] text-gray-500 truncate">
                                    {conn.db_host}:{conn.db_port}/{conn.db_name}
                                  </p>
                                </div>
                              </div>
                              {currentConnection?.database_id === conn.database_id && (
                                <i className="fas fa-check text-blue-600 text-xs"></i>
                              )}
                            </div>
                          </button>
                        ))}
                      </div>
                    </div>
                  ))}

                  {/* 分隔线 */}
                  <div className="border-t border-gray-100 my-2"></div>

                  {/* 管理连接 */}
                  <button
                    onClick={() => {
                      setShowMenu(false)
                      router.push('/dashboard/connections')
                    }}
                    className="w-full px-3 py-2 rounded-md text-left hover:bg-gray-50 transition-colors"
                  >
                    <div className="flex items-center space-x-2">
                      <i className="fas fa-cog text-xs text-gray-400"></i>
                      <span className="text-xs text-gray-700">管理连接</span>
                    </div>
                  </button>
                </>
              )}
            </div>
          </div>
        </>
      )}
    </div>
  )
}

