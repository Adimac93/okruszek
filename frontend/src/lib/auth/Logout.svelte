<script lang="ts">
    import { Fetch } from "../api";
    import { isAuthorized } from "../stores";

    const api = new Fetch();
    const logout = async () => {
        const res = await api.json("/api/auth/logout", "POST");
        if (res) {
            isAuthorized.set(false);
        }
    };
</script>

<button on:click={logout}>Logout</button>
