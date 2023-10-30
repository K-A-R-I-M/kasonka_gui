<script lang="ts">
  import "../app.css";
  import Footer from "$lib/Footer.svelte";
  import Header from "$lib/Header.svelte";
  import { invoke } from "@tauri-apps/api/tauri";
  import { setContext, onDestroy, onMount } from "svelte";
  import Notification from "$lib/Notification.svelte";
  import type { Notif, Payload } from "$lib/types";

  import { fade, fly } from "svelte/transition";
  import { flip } from "svelte/animate";

  let current_title_audio = "Audio Title";
  let play = false;
  let notifications: Array<Notif> = [];

  setContext("cta", { update_cta, update_play, notify });

  function update_cta(newCta: string) {
    current_title_audio = newCta;
  }
  function update_play(new_play: boolean) {
    play = new_play;
  }
  function notify(audio_title: string) {
    const newNotif: Notif = {
      id: notifications.length + 1 + Date.now(),
      audio_title: audio_title,
    };
    notifications = [...notifications, newNotif];

    setTimeout(() => {
      removeNotification(newNotif.id);
    }, 2500);
  }

  function removeNotification(id: number) {
    // Filter out the notification with the specified ID
    notifications = notifications.filter(
      (notification) => notification.id !== id
    );
  }

  let intervalId: number;

  async function startSerialEventListener() {
    current_title_audio = current_title_audio = await invoke("get_cta");
  }

  // Start checking the API when the component is mounted
  onMount(() => {
    startSerialEventListener(); // Initial fetch
    intervalId = setInterval(startSerialEventListener, 5000); // Fetch data every 5 seconds (adjust as needed)
  });

  // Stop checking the API when the component is destroyed
  onDestroy(() => {
    clearInterval(intervalId);
  });
</script>

<div class="flex flex-col bg-black text-white min-h-screen">
  <div
    class="fixed top-0 p-4 w-52 h-72 mt-8 overflow-y-hidden overflow-x-hidden"
  >
    <div class="flex flex-col-reverse justify-center items-center">
      {#each notifications as notification (notification.id)}
        <div in:fly={{ y: -288 }} out:fade animate:flip={{ duration: 200 }}>
          <Notification audio_title={notification.audio_title} />
        </div>
      {/each}
    </div>
  </div>
  <Header />
  <div class="flex-grow mb-28 mt-12">
    <slot />
  </div>
  <Footer {current_title_audio} {play} />
</div>
