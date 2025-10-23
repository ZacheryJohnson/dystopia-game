<script setup lang="ts">
import { computed, inject, onMounted, type Ref, ref } from 'vue';
import {
    type GetProposalsResponse,
    Proposal as ProposalT,
    ProposalOption,
} from '%/services/vote/vote.ts';
import Proposal from '@/components/Proposal.vue';
import { getAuthStore } from '@/stores/Auth.ts';
import { fetchApi } from '@/utils.ts';

type TempProposalOptionT = {
    id: number,
    description: string,
    name: string,
    effects: any[], // ignored for now, ideally we don't show this in UI
};

type TempProposalT = {
    id: number,
    name: string,
    description: string,
    options: TempProposalOptionT[],
};

const proposals: Ref<TempProposalT[] | null> = ref(null);
const hasProposals = computed(() => proposals.value && proposals.value.length > 0);
const authStore = getAuthStore();
const isAuthed = computed(/*() => authStore.cookie.length > 0*/ () => true);

onMounted(async () => {
    // ZJ-TODO: don't use intermediate type
    const tempProposals: TempProposalT[] = (await (
        await fetchApi('vote/proposal')
    ).json())['proposals'];

    proposals.value = tempProposals;
});
</script>

<template>
    <main>
        <div v-if="hasProposals && isAuthed">
            <h1>Proposals</h1>
            <div id="proposals" v-for="proposal of proposals">
                <Proposal :key="proposal.id" :proposal="proposal" />
            </div>
        </div>
        <div v-else>
            <article style="text-align: center; width: 50%; margin-left: auto; margin-right: auto">
                <p>DAX is a work in progress project, and many features are not implemented.</p>
                <p>
                    Click a match result at the top to see a visualization of how the match played
                    out.
                </p>
            </article>
        </div>
    </main>
</template>

<style scoped>
main {
    overflow: clip;
}

#proposals {
    padding-left: 5px;
}
</style>
