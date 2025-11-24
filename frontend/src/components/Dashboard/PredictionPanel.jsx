import React from 'react';

export default function PredictionPanel({ predictions }) {
  return (
    <div className="bg-white shadow rounded-lg p-6 mb-8">
      <h3 className="text-lg font-medium text-gray-900 mb-4">Predictions</h3>
      {Array.isArray(predictions) && predictions.length > 0 ? (
        <ul>
          {predictions.map((prediction, idx) => (
            <li key={idx} className="mb-2">
              {typeof prediction === 'object'
                ? JSON.stringify(prediction)
                : String(prediction)}
            </li>
          ))}
        </ul>
      ) : (
        <div className="text-gray-500">No predictions available.</div>
      )}
    </div>
  );
}