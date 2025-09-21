<template>
  <div class="w-full h-16 relative overflow-hidden">
    <div ref="box" class="h-16">
      <Line ref="lineChart" :data="chartData" :options="chartOptions"/>
    </div>

    <!-- Tx/Rx labels on the right -->
    <div
        class="absolute top-2 right-5 flex flex-col items-end space-y-1 text-sm text-gray-700 bg-white/80 px-1 py-1 rounded">
      <div class="font-semibold"
           style="color: rgba(59,130,246,0.8)">Tx: {{ tx_avg }}
      </div>
      <div class="font-semibold"
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
    const tx_avg = ref("? b/s");
    const rx_avg = ref("? b/s");
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
      animation: false,
      plugins: {},
      scales: {x: {display: false}, y: {display: false, beginAtZero: true}},
    };

    const box = ref(null);
    let intervalId = null;
    let pos = 0;

    // move function
    function startMove(ratio, duration = 1000) {
      if (intervalId) return; // prevent multiple timers
      const refresh_interval = 20;
      intervalId = setInterval(() => {
        if (box.value) {
          pos += 1;
          const total_margin = ratio * box.value.getBoundingClientRect().width
          box.value.style.marginLeft = `${Math.round(-2 * total_margin)}px`;
          const total_transform = total_margin * (1 - pos / (duration / refresh_interval));
          box.value.style.transform = `translateX(${Math.round(total_transform)}px)`;
        }
      }, refresh_interval); // every interval milliseconds
    }

// reset function
    function resetMove(ratio) {
      if (intervalId) {
        clearInterval(intervalId);
        intervalId = null;
      }
      pos = 0;
      if (box.value) {
        const total_margin = ratio * box.value.getBoundingClientRect().width
        box.value.style.marginLeft = `${Math.round(-2 * total_margin)}px`;
        box.value.style.transform = `translateX(${Math.round(total_margin)}px)`;
      }
    }

    watch(() => props.telemetry, (newTelemetry) => {
          if (Object.keys(newTelemetry.data).length < 2) {
            tx_avg.value = "? b/s";
            rx_avg.value = "? b/s";
            return;
          }
          const txs = [];
          const rxs = [];
          const timestamps = [];
          let prev_telem_data = {};

          for (const telem_data of newTelemetry.data) {
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

            txs.push(tx / ts);
            rxs.push(rx / ts);
            timestamps.push(telem_data.timestamp);
            prev_telem_data = telem_data;

          }
          tx_avg.value = formatThroughputBits(txs.at(-1));
          rx_avg.value = formatThroughputBits(rxs.at(-1));

          if (Object.keys(newTelemetry.data).length < 2) return;
          let latest_timestamp_dt = timestamps.at(-1) - timestamps.at(-2);
          while (txs.length < newTelemetry.max_len - 1) {
            txs.unshift(0);
            rxs.unshift(0);
            timestamps.unshift(timestamps[0] - latest_timestamp_dt);
          }

          // Update the chart directly
          const chart = lineChart.value.chart;
          chart.data.labels = timestamps;
          chart.data.datasets[0].data = rxs;
          chart.data.datasets[1].data = txs;
          let ratio_new_data = latest_timestamp_dt / (timestamps.at(-1) - timestamps[0]);
          resetMove(ratio_new_data)
          chart.update();
          startMove(ratio_new_data);
        },
        {deep: true}
    );

    return {chartData, chartOptions, lineChart, tx_avg, rx_avg, box};
  }
});
</script>
