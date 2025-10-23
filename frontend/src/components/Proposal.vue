<script setup lang="ts">
import { computed, onMounted, type Ref, ref } from 'vue';
import { Proposal, VoteOnProposalRequest } from '%/services/vote/vote.ts';
import { fetchApi } from '@/utils.ts';

const props = defineProps(['proposal']);

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

const proposal: Ref<TempProposalT | null> = ref(null);
const votedOnOption: Ref<number | null> = ref(null);
const hasVoted = computed(() => votedOnOption.value !== null);

const sendVote = async (proposalId: number, optionId: number) => {
    const request = VoteOnProposalRequest.create();
    request.proposalId = proposalId;
    request.optionId = optionId;

    const response = await fetchApi('vote/submit', {
        method: 'POST',
        body: JSON.stringify({
            "proposal_id": proposalId,
            "proposal_option_id": optionId,
        }),
    });

    if (response.status === 200) {
        votedOnOption.value = optionId;
    }
};

onMounted(async () => {
    proposal.value = props.proposal;
});
</script>

<template>
    <div>
        <p>
            <strong>{{ proposal?.name }}</strong>
        </p>
        <p>{{ proposal?.description }}</p>
        <button
            v-for="option in proposal?.options"
            @click="sendVote(proposal!.id!, option!.id!)"
            :key="option.id"
            :disabled="hasVoted"
            :class="[votedOnOption === option.id ? 'chosen' : '']"
        >
            {{ option.name }}
        </button>
    </div>
</template>

<style scoped>
div {
    border-style: dashed;
    border-width: 1px;
    padding-left: 5px;
}

div button:disabled.chosen {
    background-color: lightgreen;
}
</style>
