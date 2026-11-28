// Diálogos sobre o projeto: Sobre, verificação de atualizações, alertas
// publicados, relato de problema e página para contribuir.

import * as api from "../api";
import { $ } from "../dom";
import { setStatus } from "./status";

export function mountProjectDialogs(info: { version: string; license: string }) {
  const aboutDialog = $<HTMLDialogElement>("about");
  const reportDialog = $<HTMLDialogElement>("report");
  const reportError = $("report-error");
  const updates = $<HTMLDialogElement>("updates");
  const updateStatus = $("update-status");
  const updateNotes = $("update-notes");
  const updateDownload = $<HTMLButtonElement>("update-download");
  let updateRequest = 0;
  let alertRequest = 0;

  $("home-version").textContent = `v${info.version}`;
  $("about-version").textContent = `Versão ${info.version}`;
  $("about-license").textContent = info.license;
  $("home-about").onclick = () => aboutDialog.showModal();

  $("about-updates").onclick = async () => {
    const request = ++updateRequest;
    updateNotes.hidden = true;
    updateDownload.textContent = "Ver versões no GitHub";
    updateStatus.textContent = "Verificando atualizações…";
    $("update-error").hidden = true;
    updates.showModal();
    try {
      const result = await api.checkUpdates();
      if (request !== updateRequest) return;
      if (result.status === "available") {
        updateStatus.textContent = `Versão ${result.latest} disponível. Versão instalada ${result.current}.`;
        updateNotes.textContent = result.notes;
        updateNotes.hidden = !result.notes;
        updateDownload.textContent = "Baixar atualização";
      } else if (result.status === "current") {
        updateStatus.textContent = `O Ferro Studio ${result.current} está atualizado.`;
      } else {
        updateStatus.textContent = "Ainda não há uma versão estável publicada no GitHub.";
      }
    } catch (message) {
      if (request === updateRequest) {
        updateStatus.textContent = typeof message === "string" ? message : "Não foi possível verificar atualizações.";
      }
    }
  };
  updateDownload.onclick = async () => {
    updateDownload.disabled = true;
    $("update-error").hidden = true;
    try {
      await api.openProjectPage("releases");
    } catch {
      $("update-error").hidden = false;
    } finally {
      updateDownload.disabled = false;
    }
  };

  $("home-alerts").onclick = async () => {
    const request = ++alertRequest;
    const dialog = $<HTMLDialogElement>("alerts");
    const status = $("alerts-status");
    const list = $("alerts-list");
    status.hidden = false;
    status.textContent = "Consultando alertas…";
    list.hidden = true;
    list.replaceChildren();
    dialog.showModal();
    try {
      const alerts = await api.projectAlerts();
      if (request !== alertRequest) return;
      if (!alerts.length) {
        status.textContent = "Não há alertas ou avisos.";
        return;
      }
      for (const alert of alerts) {
        const article = document.createElement("article");
        const title = document.createElement("h3");
        title.textContent = alert.title;
        const message = document.createElement("p");
        message.textContent = alert.message;
        article.append(title, message);
        list.append(article);
      }
      status.hidden = true;
      list.hidden = false;
    } catch (message) {
      if (request === alertRequest) {
        status.textContent = typeof message === "string" ? message : "Não foi possível consultar os alertas.";
      }
    }
  };

  $("home-report").onclick = $("about-report").onclick = () => {
    reportError.hidden = true;
    reportDialog.showModal();
  };
  $("home-contribute").onclick = () => {
    void api.openProjectPage("contribute").catch(() => setStatus("Não foi possível abrir o navegador.", "error"));
  };
  $("report-open").onclick = async () => {
    const button = $<HTMLButtonElement>("report-open");
    button.disabled = true;
    reportError.hidden = true;
    try {
      await api.openProjectPage("bug_report");
      reportDialog.close();
    } catch {
      reportError.textContent = "Não foi possível abrir o navegador. Use github.com/wendelmax/ferro-studio/issues/new.";
      reportError.hidden = false;
    } finally {
      button.disabled = false;
    }
  };

  return {
    showAbout: () => aboutDialog.showModal(),
  };
}
