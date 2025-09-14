<template>
  <div class="w-full h-16 relative">
    <Line ref="lineChart" :data="chartData" :options="chartOptions"/>

    <!-- Tx/Rx labels on the right -->
    <div
        class="absolute top-2 right-5 flex flex-col items-end space-y-1 text-sm text-gray-700 bg-white/80 px-1 py-1 rounded">
      <div class="font-semibold font-mono"
           style="color: rgba(59,130,246,0.8)">Tx: {{ tx_avg }}
      </div>
      <div class="font-semibold font-mono"
           style="color: rgba(34,197,94,0.8)">Rx: {{ rx_avg }}
      </div>
    </div>
  </div>
</template>

<script>
import {defineComponent, ref, watch} from "vue";
import {
  Chart as ChartJS,
  LineElement,
  LinearScale,
  PointElement,
  CategoryScale,
  Filler
} from "chart.js";
import {Line} from "vue-chartjs";

function formatThroughputBits(bytesPerSec) {
  if (!isFinite(bytesPerSec) || bytesPerSec <= 0) return "0 b/s";

  const factor = 1000; // SI scaling
  let value = bytesPerSec * 8.; // convert to bits/s

  const units = ["b/s", "kb/s", "Mb/s", "Gb/s", "Tb/s"];

  let unitIndex = 0;
  while (value >= factor && unitIndex < units.length - 1) {
    value /= factor;
    unitIndex++;
  }

  // round to 1 decimal, drop trailing .0
  const rounded = Math.round(value * 10) / 10;
  const text = rounded % 1 === 0 ? String(rounded) : rounded.toFixed(1);

  return `${text} ${units[unitIndex]}`;
}

ChartJS.register(
    LineElement,
    LinearScale,
    PointElement,
    CategoryScale,
    Filler
);

export default defineComponent({
  name: "TrafficGraph",
  props: {
    network: {
      type: Object,
      default: {},
    },
    telemetry: {
      type: Object,
      default: {},
    },
  },
  components: {Line},
  setup(props) {
    const tx_avg = ref(formatThroughputBits(0));
    const rx_avg = ref(formatThroughputBits(0));
    const lineChart = ref(null);

    const chartData = ref({
      labels: [0],
      datasets: [
        {
          label: "RX",
          borderColor: "rgba(34,197,94,0.2)", // faint green
          backgroundColor: "rgba(34,197,94,0.05)",
          data: [0],
          tension: 0.3,
          fill: "origin",
          pointRadius: 0,
        },
        {
          label: "TX",
          borderColor: "rgba(59,130,246,0.2)", // faint blue
          backgroundColor: "rgba(59,130,246,0.05)",
          data: [0],
          tension: 0.3,
          fill: "origin",
          pointRadius: 0,
        },
      ],
    });

    const chartOptions = {
      responsive: true,
      maintainAspectRatio: false,
      animation: {
        duration: 500, // smooth animation
        easing: "linear",
      },
      plugins: {},
      scales: {x: {display: false}, y: {display: false, beginAtZero: true}},
    };

    watch(() => props.telemetry, (newTelemetry) => {
          if (!newTelemetry || Object.keys(newTelemetry).length < 2) return;

          const txs = [];
          const rxs = [];
          const timestamps = [];
          let prev_telem_data = {};

          for (const telem_data of newTelemetry) {
            if (Object.keys(prev_telem_data).length === 0) {
              prev_telem_data = telem_data;
              continue;
            }

            let tx = 0,
                rx = 0;
            const ts = (telem_data.timestamp - prev_telem_data.timestamp) / 1000;

            for (const [connection_id, telemetry_details] of Object.entries(telem_data.datum)) {
              if (!prev_telem_data.datum[connection_id]) continue;

              if (connection_id.startsWith(props.network.this_peer)) {
                tx += telemetry_details.transfer_a_to_b - prev_telem_data.datum[connection_id].transfer_a_to_b;
                rx += telemetry_details.transfer_b_to_a - prev_telem_data.datum[connection_id].transfer_b_to_a;
              } else if (connection_id.endsWith(props.network.this_peer)) {
                tx += telemetry_details.transfer_b_to_a - prev_telem_data.datum[connection_id].transfer_b_to_a;
                rx += telemetry_details.transfer_a_to_b - prev_telem_data.datum[connection_id].transfer_a_to_b;
              }
            }

            txs.push((tx / ts) * 8);
            rxs.push((rx / ts) * 8);
            timestamps.push(telem_data.timestamp);
            prev_telem_data = telem_data;

          }
          // let tx_sum = 0;
          // let rx_sum = 0;
          // let ts_sum = 0;
          // let counter = 0;
          // const zipped = txs.map((val, i) => [val, rxs[i], timestamps[i]]);
          // zipped.reverse()
          // for (const [tx, rx, ts] of zipped) {
          //   // get the average speed of the ~last two seconds
          //   if (ts_sum > 2.5) continue;
          //   ts_sum += ts;
          //   tx_sum += tx;
          //   rx_sum += rx;
          //   counter += 1;
          // }

          tx_avg.value = formatThroughputBits(txs[txs.length - 1] / 8.);
          rx_avg.value = formatThroughputBits(rxs[rxs.length - 1] / 8.);
          // tx_avg.value = formatThroughputBits(tx_sum / counter / 8.);
          // rx_avg.value = formatThroughputBits(rx_sum / counter / 8.);

          // Update chart directly
          const chart = lineChart.value.chart;
          chart.data.labels = timestamps;
          chart.data.datasets[0].data = rxs;
          chart.data.datasets[1].data = txs;
          chart.update();
        },
        {deep: true}
    );

    return {chartData, chartOptions, lineChart, tx_avg, rx_avg};
  }
});
</script>
