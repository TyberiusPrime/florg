<script lang="ts">
  export let params;

  var listener = new window.keypress.Listener();
  listener.register_combo({
    keys: "esc",
    is_unordered: true,
    exclusive: true,
    prevent_default: true,
    prevent_repeat: true,
    on_keyup: async (e, count, repeated) => {
      window.history.back();
    },
  });
  listener.register_combo({
    keys: "enter",
    prevent_repeat: true,
    prevent_default: true,
    on_keyup: async (ev) => {
      window.location.href = "/";
    },
  });
</script>

<div>
    <h1>not found!</h1>
    Press <span class="hotkey">esc</span> to go
    <a on:click={() => window.history.back()}> back</a>, Press
    <span class="hotkey">Enter</span> to start again at <a href="/">root</a>.
</div>
