<script lang="ts">
  import CardList from "$lib/CardList.svelte";
  import { invoke } from "@tauri-apps/api/tauri";
  import type { AudioKa } from "$lib/types";
  import { getContext, onDestroy, onMount } from "svelte";
  const { str_to_time }: any = getContext("cta");

  let listAudios: AudioKa[] = [];
  let intervalId: number;

  async function loadList() {
    let audio_title_processed: Map<string, string> = await invoke(
      "get_list_audio"
    );
    if (audio_title_processed.size >= 0) {
      audio_title_processed.forEach((audio_title: string, time: string) => {
        const newAudio: AudioKa = {
          title: audio_title,
          duration: time,
          duration_nb: str_to_time(time),
        };

        listAudios = [...listAudios, newAudio];
      });
    }
  }

  async function updateList() {
    let tmplistAudios: AudioKa[] = [];
    let list_audio_processed: AudioKa[] = await invoke("get_list_audio");
    if (list_audio_processed.length >= 0) {
      list_audio_processed.forEach((audio: AudioKa) => {
        const newAudio: AudioKa = {
          title: audio.title,
          duration: audio.duration,
          duration_nb: str_to_time(audio.duration),
        };

        tmplistAudios = [...tmplistAudios, newAudio];
      });
    }

    listAudios = tmplistAudios;
  }

  // Start checking the API when the component is mounted
  onMount(() => {
    updateList(); // Initial fetch
    intervalId = setInterval(updateList, 1500); // Fetch data every 5 seconds (adjust as needed)
  });

  // Stop checking the API when the component is destroyed
  onDestroy(() => {
    clearInterval(intervalId);
  });

  if (typeof window !== "undefined") {
    $: loadList();
  }
</script>

<div class="overflow-auto pb-5 mb-5">
  <CardList {listAudios} />
</div>
