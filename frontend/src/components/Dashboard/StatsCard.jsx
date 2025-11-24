import React from 'react';

export default function StatsCard({ title, value, icon: Icon, color, trend }) {
  return (
    <div className={`bg-white shadow rounded-lg p-6 flex flex-col items-start border-l-4 border-${color}-500`}>
      <div className="flex items-center mb-2">
        {Icon && <Icon className={`h-6 w-6 text-${color}-500 mr-2`} />}
        <h4 className="text-lg font-semibold text-gray-900">{title}</h4>
      </div>
      <div className="text-2xl font-bold text-gray-800">{value}</div>
      {trend !== undefined && (
        <div className="mt-1 text-sm text-gray-500">
          {trend > 0 ? '▲' : trend < 0 ? '▼' : ''} {Math.abs(trend || 0)}%
        </div>
      )}
    </div>
  );
}