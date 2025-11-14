'use client'

import { usePathname } from 'next/navigation'

const pageTitles: Record<string, string> = {
  '/dashboard': '仪表盘',
  '/dashboard/schema': 'Schema 浏览器',
  '/dashboard/tables': '数据表管理',
  '/dashboard/query': 'SQL 查询器',
  '/dashboard/transaction': '事务管理',
  '/dashboard/users': '用户管理',
}

export default function Header() {
  const pathname = usePathname()
  const title = pageTitles[pathname] || '管理后台'

  return (
    <div className="bg-white border-b border-gray-100 shadow-sm">
      <div className="px-6 py-4 flex items-center justify-between">
        <div className="flex items-center space-x-4">
          <div className="flex items-center space-x-2 text-sm text-gray-600">
            <i className="fas fa-home text-xs"></i>
            <i className="fas fa-chevron-right text-xs text-gray-400"></i>
            <span className="font-medium text-gray-900">{title}</span>
          </div>
        </div>

        <div className="flex items-center space-x-3">
          <div className="flex items-center space-x-2 px-3 py-1.5 bg-gray-50 rounded-lg border border-gray-200">
            <div className="w-2 h-2 bg-green-500 rounded-full animate-pulse"></div>
            <span className="text-xs text-gray-600 font-mono">localhost:3000</span>
          </div>

          <button className="btn-success shadow-sm hover:shadow-md transform transition-all duration-200 hover:-translate-y-0.5">
            <i className="fas fa-heart-pulse mr-1.5"></i>
            健康检查
          </button>
        </div>
      </div>
    </div>
  )
}

