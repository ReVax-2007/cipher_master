import { Client, Events, GatewayIntentBits, MessageFlags } from "discord.js";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";
import { randomInt } from "node:crypto";
import { createChallenge, describeKey, hintFor } from "./ciphers.js";
import { ChallengeStore } from "./store.js";

if (!process.env.DISCORD_TOKEN) {
  console.error("Set DISCORD_TOKEN before starting the bot (see README).");
  process.exit(1);
}

const here = dirname(fileURLToPath(import.meta.url));
const quotes = JSON.parse(readFileSync(resolve(here, "../../src-tauri/data/quotes.json"), "utf8"));
const store = new ChallengeStore(process.env.CHALLENGE_STORE ?? resolve(here, "data/challenges.json"));
const client = new Client({ intents: [GatewayIntentBits.Guilds] });
const ephemeral = { flags: MessageFlags.Ephemeral };

client.once(Events.ClientReady, () => console.log(`Cipher Master connected as ${client.user.tag}`));
client.on("interactionCreate", async (interaction) => {
  if (!interaction.isChatInputCommand()) return;
  const id = `${interaction.guildId ?? "dm"}:${interaction.user.id}`;
  try {
    if (interaction.commandName === "cipher") {
      const type = interaction.options.getString("type") ?? "caesar";
      const challenge = createChallenge(quotes[randomInt(quotes.length)], type);
      store.set(id, challenge);
      await interaction.reply({ content: `**${challenge.label.toUpperCase()} CHALLENGE**\n\`\`\`\n${challenge.text}\n\`\`\`\nUse \`/hint\` or \`/solution\` (only you will see them).` });
    } else if (interaction.commandName === "hint" || interaction.commandName === "solution") {
      const challenge = store.get(id);
      if (!challenge) return interaction.reply({ content: "Create a challenge first with `/cipher`.", ...ephemeral });
      const content = interaction.commandName === "hint"
        ? `Cipher: **${challenge.label}**. ${hintFor(challenge)}`
        : `Plaintext: **${challenge.plaintext}**\nKey: **${describeKey(challenge)}**`;
      await interaction.reply({ content, ...ephemeral });
    }
  } catch (error) {
    console.error(error);
    if (!interaction.replied) await interaction.reply({ content: "Something went wrong creating that challenge.", ...ephemeral });
  }
});

client.login(process.env.DISCORD_TOKEN);
