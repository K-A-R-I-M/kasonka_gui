<script lang="ts">
  import "../app.css";
  import Footer from "$lib/Footer.svelte";
  import Header from "$lib/Header.svelte";
  import { invoke } from "@tauri-apps/api/tauri";
  import { setContext, onDestroy, onMount } from "svelte";
  import Notification from "$lib/Notification.svelte";
  import type { AudioKa, Notif } from "$lib/types";

  import { fade, fly } from "svelte/transition";
  import { flip } from "svelte/animate";

  let current_audio: AudioKa;
  let current_audio_inside: AudioKa = {
    title: "",
    duration: "",
    duration_nb: 0,
  };
  let play = false;
  let notifications: Array<Notif> = [];
  let intervalId: number;
  let intervalId2: number;

  setContext("cta", {
    update_cta,
    update_play,
    notify,
    str_to_time,
    get_cta_from_back,
    time_to_str,
  });

  function update_cta(newCta: AudioKa) {
    current_audio = newCta;
  }
  function update_play(new_play: boolean) {
    play = new_play;
  }
  function notify(title: string, text: string) {
    const newNotif: Notif = {
      id: notifications.length + 1 + Date.now(),
      title: title,
      text: text,
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

  function str_to_time(time_str: string) {
    if (time_str !== undefined) {
      let hours: string = time_str.split(":")[0];
      let min: string = time_str.split(":")[1];
      let sec: string = time_str.split(":")[2];

      return (
        Number.parseInt(hours) * 60 * 60 +
        Number.parseInt(min) * 60 +
        Number.parseInt(sec)
      );
    } else {
      return 0;
    }
  }

  function time_to_str(time: number) {
    let time_str: string;
    let hours_str = "00";
    let minutes_str = "00";
    let secondes_str = "00";
    let hours = time / 60 / 60 - ((time / 60 / 60) % 1);
    let minutes = time / 60 - ((time / 60) % 1);
    while (minutes >= 60) {
      minutes = minutes / 60;
    }
    let secondes = time % 60;

    if (hours < 10) {
      hours_str = "0" + hours;
    } else {
      hours_str = hours.toString();
    }

    if (minutes < 10) {
      minutes_str = "0" + minutes;
    } else {
      minutes_str = minutes.toString();
    }

    if (secondes < 10) {
      secondes_str = "0" + secondes;
    } else {
      secondes_str = secondes.toString();
    }

    return hours_str + ":" + minutes_str + ":" + secondes_str;
  }

  async function get_cta_from_back() {
    current_audio = await invoke("get_cta");
    if (
      current_audio.title !== current_audio_inside.title &&
      current_audio.duration !== current_audio_inside.duration
    ) {
      current_audio_inside = current_audio;
      notify("Now playing", 'Start to play: "' + current_audio.title + '"');
    }
    return current_audio;
  }

  async function startSerialEventListener() {
    current_audio = await get_cta_from_back();
  }

  async function playStatusChecker() {
    play = await invoke("get_player_status");
  }

  // Start checking the API when the component is mounted
  onMount(() => {
    startSerialEventListener(); // Initial fetch
    intervalId = setInterval(startSerialEventListener, 2000); // Fetch data every 5 seconds (adjust as needed)
    playStatusChecker();
    intervalId2 = setInterval(playStatusChecker, 500);
  });

  // Stop checking the API when the component is destroyed
  onDestroy(() => {
    clearInterval(intervalId);
    clearInterval(intervalId2);
  });
</script>

<div class="flex flex-col bg-black text-white min-h-screen">
  <div
    class="fixed top-0 p-4 w-52 h-72 mt-16 overflow-y-hidden overflow-x-hidden"
  >
    <div class="flex flex-col-reverse justify-center items-center">
      {#each notifications as notification (notification.id)}
        <div in:fly={{ y: -288 }} out:fade animate:flip={{ duration: 200 }}>
          <Notification title={notification.title} text={notification.text} />
        </div>
      {/each}
    </div>
  </div>
  <Header {current_audio} />
  <div class="flex-grow mb-28 mt-20">
    <slot />
  </div>
  <Footer {current_audio} {play} />
</div>
