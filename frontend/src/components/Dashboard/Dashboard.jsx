import React, { useState } from 'react';
import { 
  DollarSign, 
  Activity, 
  TrendingUp, 
  AlertTriangle,
  RefreshCw,
  Download
} from 'lucide-react';
import { useAPIData } from '../../hooks/useAPIData';
import { usePredictions } from '../../hooks/usePredictions';
import StatsCard from './StatsCard';
import PredictionPanel from './PredictionPanel';
import CostChart from '../Charts/CostChart';
import TokenChart from '../Charts/TokenChart';
import RequestChart from '../Charts/RequestChart';
import { formatCurrency, formatNumber } from '../../utils/formatters';

export default function Dashboard() {
  const [period, setPeriod] = useState('7d');
  const [autoRefresh, setAutoRefresh] = useState(false);
  
  const { data, stats, loading, error, refetch } = useAPIData(period, autoRefresh);
  const { predictions, loading: predictionsLoading } = usePredictions();

  const handleExport = async () => {
    try {
      const blob = await api.exportUsage({ period });
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `usage-export-${Date.now()}.json`;
      a.click();
    } catch (error) {
      console.error('Export failed:', error);
    }
  };

  if (error) {
    return (
      <div className="min-h-screen bg-gray-50 flex items-center justify-center">
        <div className="text-center">
          <AlertTriangle className="mx-auto h-12 w-12 text-red-500" />
          <h2 className="mt-4 text-xl font-semibold text-gray-900">Error Loading Data</h2>
          <p className="mt-2 text-gray-600">{error}</p>
          <button
            onClick={refetch}
            className="mt-4 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
          >
            Try Again
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-50">
      {/* Header */}
      <div className="bg-white shadow">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="py-6 md:flex md:items-center md:justify-between">
            <div className="flex-1 min-w-0">
              <h1 className="text-3xl font-bold text-gray-900">
                📊 API Usage Dashboard
              </h1>
              <p className="mt-1 text-sm text-gray-500">
                Track and analyze your API usage in real-time
              </p>
            </div>
            <div className="mt-4 flex md:mt-0 md:ml-4 space-x-3">
              <button
                onClick={() => setAutoRefresh(!autoRefresh)}
                className={`inline-flex items-center px-4 py-2 border rounded-lg text-sm font-medium ${
                  autoRefresh
                    ? 'bg-blue-600 text-white border-blue-600'
                    : 'bg-white text-gray-700 border-gray-300 hover:bg-gray-50'
                }`}
              >
                <RefreshCw className="h-4 w-4 mr-2" />
                Auto-refresh
              </button>
              <button
                onClick={handleExport}
                className="inline-flex items-center px-4 py-2 border border-gray-300 rounded-lg text-sm font-medium text-gray-700 bg-white hover:bg-gray-50"
              >
                <Download className="h-4 w-4 mr-2" />
                Export
              </button>
            </div>
          </div>
        </div>
      </div>

      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {/* Period Selector */}
        <div className="mb-6 flex space-x-2">
          {['24h', '7d', '30d', '90d'].map((p) => (
            <button
              key={p}
              onClick={() => setPeriod(p)}
              className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
                period === p
                  ? 'bg-blue-600 text-white'
                  : 'bg-white text-gray-700 hover:bg-gray-50'
              }`}
            >
              {p === '24h' ? 'Last 24 Hours' : 
               p === '7d' ? 'Last 7 Days' :
               p === '30d' ? 'Last 30 Days' : 'Last 90 Days'}
            </button>
          ))}
        </div>

        {loading ? (
          <div className="flex justify-center items-center h-64">
            <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
          </div>
        ) : (
          <>
            {/* Stats Cards */}
            <div className="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-4 mb-8">
              <StatsCard
                title="Total Cost"
                value={formatCurrency(stats?.total_cost || 0)}
                icon={DollarSign}
                color="blue"
                trend={stats?.cost_trend}
              />
              <StatsCard
                title="Total Tokens"
                value={formatNumber(stats?.total_tokens || 0)}
                icon={Activity}
                color="purple"
                trend={stats?.token_trend}
              />
              <StatsCard
                title="Total Requests"
                value={formatNumber(stats?.total_requests || 0)}
                icon={TrendingUp}
                color="green"
                trend={stats?.request_trend}
              />
              <StatsCard
                title="Error Rate"
                value={`${stats?.error_rate?.toFixed(2) || 0}%`}
                icon={AlertTriangle}
                color="red"
                trend={stats?.error_trend}
              />
            </div>

            {/* Predictions */}
            {!predictionsLoading && predictions && (
              <PredictionPanel predictions={predictions} />
            )}

            {/* Charts */}
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-8">
              <CostChart data={data} />
              <TokenChart data={data} />
            </div>

            <div className="mb-8">
              <RequestChart data={data} />
            </div>

            {/* Recent Usage Table */}
            <div className="bg-white shadow rounded-lg overflow-hidden">
              <div className="px-6 py-4 border-b border-gray-200">
                <h3 className="text-lg font-medium text-gray-900">
                  Recent Usage
                </h3>
              </div>
              <div className="overflow-x-auto">
                <table className="min-w-full divide-y divide-gray-200">
                  <thead className="bg-gray-50">
                    <tr>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Timestamp
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Model
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Tokens
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Cost
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Status
                      </th>
                    </tr>
                  </thead>
                  <tbody className="bg-white divide-y divide-gray-200">
                    {data.slice(0, 10).map((item, idx) => (
                      <tr key={idx} className="hover:bg-gray-50">
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                          {new Date(item.timestamp).toLocaleString()}
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                          {item.model_name || 'N/A'}
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                          {formatNumber(item.total_tokens)}
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                          {formatCurrency(item.cost)}
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap">
                          <span className={`px-2 inline-flex text-xs leading-5 font-semibold rounded-full ${
                            item.errors > 0
                              ? 'bg-red-100 text-red-800'
                              : 'bg-green-100 text-green-800'
                          }`}>
                            {item.errors > 0 ? 'Error' : 'Success'}
                          </span>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          </>
        )}
      </div>
    </div>
  );
}