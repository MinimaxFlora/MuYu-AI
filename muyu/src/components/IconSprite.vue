<template>
  <svg width="0" height="0" style="position: absolute" aria-hidden="true">
    <defs>
      <symbol id="i-home" viewBox="0 0 24 24"><path d="M4 10.6 12 4l8 6.6V20a1 1 0 0 1-1 1h-4.4v-6H9.4v6H5a1 1 0 0 1-1-1Z" /></symbol>
      <symbol id="i-folder" viewBox="0 0 24 24"><path d="M3 7.5A2.5 2.5 0 0 1 5.5 5h3.2l2 2h7.8A2.5 2.5 0 0 1 21 9.5v7A2.5 2.5 0 0 1 18.5 19h-13A2.5 2.5 0 0 1 3 16.5Z" /></symbol>
      <symbol id="i-film" viewBox="0 0 24 24"><rect x="3" y="5" width="18" height="14" rx="2.5" /><path d="M7 5v14M17 5v14M3 9h4M3 15h4M17 9h4M17 15h4" /></symbol>
      <symbol id="i-layers" viewBox="0 0 24 24"><path d="m12 3 9 5-9 5-9-5 9-5Z" /><path d="m3 12 9 5 9-5M3 16.5l9 5 9-5" /></symbol>
      <symbol id="i-gear" viewBox="0 0 24 24"><circle cx="12" cy="12" r="3.2" /><path d="M19.4 13.6a7.6 7.6 0 0 0 0-3.2l1.8-1.4-1.9-3.3-2.1.9a7.7 7.7 0 0 0-2.8-1.6L14 2.6h-4l-.4 2.4A7.7 7.7 0 0 0 6.8 6.6l-2.1-.9L2.8 9l1.8 1.4a7.6 7.6 0 0 0 0 3.2L2.8 15l1.9 3.3 2.1-.9c.8.7 1.8 1.3 2.8 1.6l.4 2.4h4l.4-2.4c1-.3 2-.9 2.8-1.6l2.1.9 1.9-3.3Z" /></symbol>
      <symbol id="i-plus" viewBox="0 0 24 24"><path d="M12 5v14M5 12h14" /></symbol>
      <symbol id="i-search" viewBox="0 0 24 24"><circle cx="11" cy="11" r="6.5" /><path d="m16 16 4 4" /></symbol>
      <symbol id="i-bell" viewBox="0 0 24 24"><path d="M6.5 10a5.5 5.5 0 0 1 11 0c0 4 1.5 5.5 1.5 5.5H5s1.5-1.5 1.5-5.5Z" /><path d="M10 19a2 2 0 0 0 4 0" /></symbol>
      <symbol id="i-link" viewBox="0 0 24 24"><path d="M10 14a4 4 0 0 0 5.7 0l2.3-2.3a4 4 0 0 0-5.7-5.7L11 7.3" /><path d="M14 10a4 4 0 0 0-5.7 0L6 12.3a4 4 0 0 0 5.7 5.7l1.3-1.3" /></symbol>
      <symbol id="i-play" viewBox="0 0 24 24"><path d="M8 5.5v13l11-6.5-11-6.5Z" /></symbol>
      <symbol id="i-pause" viewBox="0 0 24 24"><path d="M9 5v14M15 5v14" /></symbol>
      <symbol id="i-arrow-r" viewBox="0 0 24 24"><path d="M4 12h15m0 0-6-6m6 6-6 6" /></symbol>
      <symbol id="i-arrow-l" viewBox="0 0 24 24"><path d="M20 12H5m0 0 6-6m-6 6 6 6" /></symbol>
      <symbol id="i-chev-d" viewBox="0 0 24 24"><path d="m6 9 6 6 6-6" /></symbol>
      <symbol id="i-chev-r" viewBox="0 0 24 24"><path d="m9 6 6 6-6 6" /></symbol>
      <symbol id="i-spark" viewBox="0 0 24 24"><path d="M12 3.5 13.9 9l5.6 1.9-5.6 1.9L12 18.5l-1.9-5.7L4.5 11 10.1 9 12 3.5Z" /><path d="M18.5 16.5 19.2 18l1.6.7-1.6.6-.7 1.6-.7-1.6-1.6-.6 1.6-.7Z" /></symbol>
      <symbol id="i-check" viewBox="0 0 24 24"><path d="m5 12.5 4.5 4.5L19 6.5" /></symbol>
      <symbol id="i-clock" viewBox="0 0 24 24"><circle cx="12" cy="12" r="8" /><path d="M12 7.5V12l3 2" /></symbol>
      <symbol id="i-more" viewBox="0 0 24 24"><circle cx="6" cy="12" r="1.2" /><circle cx="12" cy="12" r="1.2" /><circle cx="18" cy="12" r="1.2" /></symbol>
      <symbol id="i-scissor" viewBox="0 0 24 24"><circle cx="6.5" cy="17" r="2.5" /><circle cx="6.5" cy="7" r="2.5" /><path d="M8.6 8.5 19 18M8.6 15.5 19 6" /></symbol>
      <symbol id="i-text" viewBox="0 0 24 24"><path d="M5 6h14M12 6v13M8.5 19h7" /></symbol>
      <symbol id="i-music" viewBox="0 0 24 24"><path d="M9 18V6l10-2v12" /><circle cx="6.5" cy="18" r="2.5" /><circle cx="16.5" cy="16" r="2.5" /></symbol>
      <symbol id="i-refresh" viewBox="0 0 24 24"><path d="M20 12a8 8 0 1 1-2.3-5.7M20 4v4.5h-4.5" /></symbol>
      <symbol id="i-trash" viewBox="0 0 24 24"><path d="M4 7h16M9 7V4.5h6V7M6.5 7l1 13h9l1-13" /></symbol>
      <symbol id="i-upload" viewBox="0 0 24 24"><path d="M12 16V6m0 0L8.5 9.5M12 6l3.5 3.5M5 18.5h14" /></symbol>
      <symbol id="i-download" viewBox="0 0 24 24"><path d="M12 4v10m0 0 3.5-3.5M12 14l-3.5-3.5M5 18.5h14" /></symbol>
      <symbol id="i-folder-open" viewBox="0 0 24 24"><path d="M3 8a2 2 0 0 1 2-2h4l2 2h7a2 2 0 0 1 2 2v1H5l-2 6Z" /><path d="m5 17 2.2-6H22l-2.2 6Z" /></symbol>
      <symbol id="i-alert" viewBox="0 0 24 24"><path d="M12 4 3 20h18L12 4Z" /><path d="M12 10v4.5M12 17.2v.3" /></symbol>
      <symbol id="i-volume" viewBox="0 0 24 24"><path d="M5 10v4h3l4 3.5v-11L8 10H5Z" /><path d="M16 9a4 4 0 0 1 0 6" /></symbol>
      <symbol id="i-expand" viewBox="0 0 24 24"><path d="M9 4H4v5M15 4h5v5M15 20h5v-5M9 20H4v-5" /></symbol>
      <symbol id="i-wave" viewBox="0 0 24 24"><path d="M3 12h2l2-5 3 10 3-12 3 14 2-7h3" /></symbol>
      <symbol id="i-save" viewBox="0 0 24 24"><path d="M5 5h11l3 3v11H5Z" /><path d="M8 5v5h7V5M8 19v-5h8v5" /></symbol>
      <symbol id="i-grid" viewBox="0 0 24 24"><rect x="4" y="4" width="6.5" height="6.5" rx="1.5" /><rect x="13.5" y="4" width="6.5" height="6.5" rx="1.5" /><rect x="4" y="13.5" width="6.5" height="6.5" rx="1.5" /><rect x="13.5" y="13.5" width="6.5" height="6.5" rx="1.5" /></symbol>
    </defs>
  </svg>
</template>
