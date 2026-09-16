import { Client, GatewayIntentBits } from "discord.js";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const quotes = JSON.parse(readFileSync(resolve(root, "src-tauri/data/quotes.json"), "utf8"));
const challenges = new Map();
const client = new Client({ intents: [GatewayIntentBits.Guilds] });

function encrypt(text, type) {
  if (type === "atbash") return text.replace(/[A-Z]/g, letter => String.fromCharCode(90 - (letter.charCodeAt(0) - 65)));
  const shift = Math.floor(Math.random() * 25) + 1;
  return { text: text.replace(/[A-Z]/g, letter => String.fromCharCode(65 + (letter.charCodeAt(0) - 65 + shift) % 26)), key: `Caesar shift ${shift}` };
}

client.once("ready", () => console.log(`Cipher Master connected as ${client.user.tag}`));
client.on("interactionCreate", async interaction => {
  if (!interaction.isChatInputCommand()) return;
  const id = `${interaction.guildId}:${interaction.user.id}`;
  if (interaction.commandName === "cipher") {
    const type = interaction.options.getString("type") ?? "caesar";
    const plaintext = quotes[Math.floor(Math.random() * quotes.length)].toUpperCase();
    const encrypted = encrypt(plaintext, type);
    const challenge = typeof encrypted === "string" ? { text: encrypted, key: "Atbash: reversed alphabet" } : encrypted;
    challenges.set(id, { ...challenge, plaintext, type });
    await interaction.reply({ content: `**${type.toUpperCase()} CHALLENGE**\n\n\`${challenge.text}\``, ephemeral: false });
  } else if (interaction.commandName === "hint" || interaction.commandName === "solution") {
    const challenge = challenges.get(id);
    if (!challenge) return interaction.reply({ content: "Create a challenge first with `/cipher`.", ephemeral: true });
    const content = interaction.commandName === "hint" ? `Cipher: **${challenge.type}**. ${challenge.type === "caesar" ? "Try counting the alphabet shift." : "Map each letter to its mirror in the alphabet."}` : `Plaintext: **${challenge.plaintext}**\nKey: **${challenge.key}**`;
    await interaction.reply({ content, ephemeral: true });
  }
});

client.login(process.env.DISCORD_TOKEN);
