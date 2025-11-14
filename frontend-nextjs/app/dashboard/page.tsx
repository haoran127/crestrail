'use client'

export default function DashboardPage() {
  return (
    <div className="space-y-6">
      <div className="card p-6">
        <h2 className="text-2xl font-semibold text-gray-800 mb-4">
          欢迎使用 CrestRail
        </h2>
        <p className="text-gray-600">
          PostgreSQL 企业级数据库管理平台
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <div className="card p-6">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-gray-600">数据库连接</p>
              <p className="text-2xl font-semibold text-gray-900 mt-2">1</p>
            </div>
            <div className="w-12 h-12 bg-primary-100 rounded-lg flex items-center justify-center">
              <i className="fas fa-database text-primary-600 text-xl"></i>
            </div>
          </div>
        </div>

        <div className="card p-6">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-gray-600">数据表</p>
              <p className="text-2xl font-semibold text-gray-900 mt-2">--</p>
            </div>
            <div className="w-12 h-12 bg-green-100 rounded-lg flex items-center justify-center">
              <i className="fas fa-table text-green-600 text-xl"></i>
            </div>
          </div>
        </div>

        <div className="card p-6">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-gray-600">查询执行</p>
              <p className="text-2xl font-semibold text-gray-900 mt-2">0</p>
            </div>
            <div className="w-12 h-12 bg-blue-100 rounded-lg flex items-center justify-center">
              <i className="fas fa-code text-blue-600 text-xl"></i>
            </div>
          </div>
        </div>

        <div className="card p-6">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-gray-600">系统状态</p>
              <p className="text-2xl font-semibold text-green-600 mt-2">正常</p>
            </div>
            <div className="w-12 h-12 bg-green-100 rounded-lg flex items-center justify-center">
              <i className="fas fa-check-circle text-green-600 text-xl"></i>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

