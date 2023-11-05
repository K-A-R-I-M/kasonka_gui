<script lang="ts">
  import ListBox from "$lib/ListBox.svelte";
  import { invoke } from "@tauri-apps/api/tauri";

  let volume_value = 0;
  let devices: string[] = [];
  let device_value: string;

  async function init() {
    device_value = await invoke("get_default_audio_device");
  }

  async function handleVolumeChange() {
    invoke("set_volume", { volumeValue: volume_value });
  }

  async function handleOutputDeviceChange(event: any) {
    device_value = await invoke("set_audio_device", {
      devnameselected: event.detail.data,
    });
  }

  async function get_volume() {
    volume_value = await invoke("get_volume");
  }

  async function get_devices() {
    devices = await invoke("get_audio_devices");
  }

  if (typeof window !== "undefined") {
    $: get_volume();
    $: get_devices();
    $: init();
  }
</script>

<div class="flex flex-col space-y-8 pt-4 px-48 mx-12">
  <div class="flex flex-col justify-center items-center">
    <label for="steps-range" class="block mb-2 text-sm font-medium text-white"
      >General Volume</label
    >

    <label for="steps-range" class="block mb-2 text-sm font-medium text-white"
      >{volume_value}</label
    >
    <input
      id="steps-range"
      type="range"
      min="0"
      max="100"
      bind:value={volume_value}
      step="1"
      on:change={handleVolumeChange}
      class="w-full h-2 rounded-lg accent-blue-700 cursor-pointer"
    />
  </div>
  <div class="flex flex-col justify-center items-center">
    <label for="select-device" class="block mb-2 text-sm font-medium text-white"
      >Output Device</label
    >
    <label for="steps-range" class="block mb-2 text-sm font-medium text-white"
      >{device_value}</label
    >
    <ListBox
      on:select={handleOutputDeviceChange}
      options={devices}
      selectedOption={device_value}
    />
  </div>
</div>
