interface Env {
    ASSETS: Fetcher;
}

export default {
    async fetch(request, env, ctx): Promise<Response> {
        return env.ASSETS.fetch(request);
    },
} satisfies ExportedHandler<Env>;