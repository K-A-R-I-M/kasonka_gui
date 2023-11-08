<script lang="ts">
  import CardList from "$lib/CardList.svelte";
  import Button from "$lib/Button.svelte";
  import { invoke } from "@tauri-apps/api/tauri";
  import type { Item } from "$lib/types";
  import { onDestroy, onMount } from "svelte";

  let listItems: Item[] = [];

  async function loadList() {
    let audio_title_processed: Map<string, string> = await invoke("get_list_audio");
    if (audio_title_processed.size >= 0) {
      audio_title_processed.forEach((audio_title: string, time: string) => {
        const newItem: Item = {
          title: audio_title,
          duration: time,
        };

        listItems = [...listItems, newItem];
      });
    }
  }

  let intervalId: number;

  async function updateList() {
    let tmplistItems: Item[] = [];
    let audio_title_processed: any[] = await invoke("get_list_audio");
    if (audio_title_processed.length >= 0) {
      console.log(audio_title_processed);
      audio_title_processed.forEach((audio: any) => {
        console.log(audio.time);
          const newItem: Item = {
            title: audio.title,
            duration: audio.time,
          };

          tmplistItems = [...tmplistItems, newItem];
      });
    }
    
    listItems = tmplistItems;
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
  <CardList {listItems} />
</div>
