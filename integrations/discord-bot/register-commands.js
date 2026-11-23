import { REST, Routes, SlashCommandBuilder } from "discord.js";
import { CIPHERS } from "./ciphers.js";

for (const name of ["DISCORD_TOKEN", "DISCORD_CLIENT_ID"]) {
  if (!process.env[name]) { console.error(`Set ${name} first (see README).`); process.exit(1); }
}

const choices = Object.entries(CIPHERS).map(([value, c]) => ({ name: c.label, value }));
const commands = [
  new SlashCommandBuilder().setName("cipher").setDescription("Create a Cipher Master challenge")
    .addStringOption((o) => o.setName("type").setDescription("Cipher to use").setRequired(false).addChoices(...choices)),
  new SlashCommandBuilder().setName("hint").setDescription("Get a private hint for your current challenge"),
  new SlashCommandBuilder().setName("solution").setDescription("Get the private solution for your current challenge"),
].map((c) => c.toJSON());

const rest = new REST({ version: "10" }).setToken(process.env.DISCORD_TOKEN);
await rest.put(Routes.applicationCommands(process.env.DISCORD_CLIENT_ID), { body: commands });
console.log("Registered /cipher, /hint, and /solution.");
