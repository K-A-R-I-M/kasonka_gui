<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import { getContext } from "svelte";
  const { notify }: any = getContext("cta");

  export let play: boolean;
  export let current_title_audio: string;

  async function resume() {
    play = await invoke("resume_pause");
  }

  async function next() {
    await invoke("next");
    await get_cta();
    play = true;
  }

  async function previous() {
    await invoke("previous");
    await get_cta();
    play = true;
  }

  async function get_cta() {
    let old_cta = current_title_audio;
    current_title_audio = await invoke("get_cta");
    if (current_title_audio != old_cta) {
      notify("Now playing", 'Start to play: "' + current_title_audio + '"');
    }
  }

  if (typeof window !== "undefined") {
    $: get_cta();
  }
</script>

<div
  class="flex fixed bottom-0 left-0 right-0 justify-around bg-gray-900 text-white text-base py-4 px-4 h-20"
>
  <div class="container m-auto">
    <div class="flex items-center justify-start">
      <p>{current_title_audio}</p>
    </div>
  </div>
  <div class="flex items-center justify-center space-x-4">
    <button
      on:click|preventDefault={previous}
      class="text-white hover:text-gray-400"
    >
      Previous
    </button>
    <button
      on:click|preventDefault={resume}
      class="bg-green-500 hover:bg-green-600 text-white font-semibold py-2 px-4 rounded-full"
    >
      {#if play}
        Pause
      {:else}
        Play
      {/if}
    </button>
    <button
      on:click|preventDefault={next}
      class="text-white hover:text-gray-400"
    >
      Next
    </button>
  </div>
  <nav class="container my-auto text-center">
    <ul class="flex justify-end space-x-4">
      <li><a href="/" class="hover:text-gray-400">Home</a></li>
      <li><a href="/playlist" class="hover:text-gray-400">Playlist</a></li>
      <li><a href="/credits" class="hover:text-gray-400">Credits</a></li>
      <li><a href="/settings" class="hover:text-gray-400">Settings</a></li>
    </ul>
  </nav>
</div>
