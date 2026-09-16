import { REST, Routes, SlashCommandBuilder } from "discord.js";

const commands = [
  new SlashCommandBuilder().setName("cipher").setDescription("Create a Cipher Master challenge").addStringOption(option => option.setName("type").setDescription("Cipher to use").setRequired(false).addChoices({ name: "Caesar", value: "caesar" }, { name: "Atbash", value: "atbash" })),
  new SlashCommandBuilder().setName("hint").setDescription("Get a private hint for your current challenge"),
  new SlashCommandBuilder().setName("solution").setDescription("Get the private solution for your current challenge")
].map(command => command.toJSON());

const rest = new REST({ version: "10" }).setToken(process.env.DISCORD_TOKEN);
await rest.put(Routes.applicationCommands(process.env.DISCORD_CLIENT_ID), { body: commands });
console.log("Registered /cipher, /hint, and /solution.");
