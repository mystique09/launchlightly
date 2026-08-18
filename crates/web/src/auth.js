(() => {
    "use strict";

    const MAX_PASSWORD_BYTES = 128;
    const passwordEncoder = new TextEncoder();

    class AuthRequestError extends Error {
        constructor(status, payload) {
            super("Authentication request failed");
            this.status = status;
            this.payload = payload;
        }
    }

    async function api(path, options = {}) {
        const request = {
            credentials: "same-origin",
            headers: { Accept: "application/json" },
            ...options,
        };

        if (options.body && typeof options.body !== "string") {
            request.headers = {
                ...request.headers,
                "Content-Type": "application/json",
            };
            request.body = JSON.stringify(options.body);
        }

        const url = path.startsWith("/api/auth/") ? path : `/api/auth${path}`;
        const response = await fetch(url, request);
        const contentType = response.headers.get("content-type") || "";
        let payload = null;
        if (contentType.includes("application/json")) {
            payload = await response.json().catch(() => null);
        }

        if (!response.ok) {
            throw new AuthRequestError(response.status, payload);
        }

        return payload;
    }

    function setBusy(form, busy) {
        form.setAttribute("aria-busy", String(busy));
        for (const control of form.querySelectorAll("input, button")) {
            control.disabled = busy;
        }

        const submit = form.querySelector("button[type='submit']");
        const label = submit?.querySelector("[data-button-label]");
        if (submit && label) {
            label.textContent = busy
                ? submit.dataset.pendingLabel
                : submit.dataset.defaultLabel;
        }
    }

    function showStatus(target, message, tone = "error", focus = tone === "error") {
        if (!target) return;
        target.textContent = message;
        target.dataset.tone = tone;
        target.hidden = false;
        if (focus) target.focus({ preventScroll: true });
    }

    function clearStatus(target) {
        if (!target) return;
        target.textContent = "";
        target.hidden = true;
        delete target.dataset.tone;
    }

    function validateMatchingPasswords(password, confirmation, message) {
        confirmation.setCustomValidity(
            confirmation.value && confirmation.value !== password.value ? message : "",
        );
    }

    function validatePasswordLength(password) {
        password.setCustomValidity(
            passwordEncoder.encode(password.value).byteLength > MAX_PASSWORD_BYTES
                ? "Password is too long. Some symbols count as more than one character."
                : "",
        );
    }

    function isInvalidCredentials(error) {
        return error instanceof AuthRequestError
            && error.status === 401
            && error.payload?.message === "Invalid credentials";
    }

    function bindSignIn() {
        const form = document.querySelector("#sign-in-form");
        if (!form) return;
        const status = document.querySelector("#sign-in-status");

        form.addEventListener("submit", async (event) => {
            event.preventDefault();
            clearStatus(status);
            if (!form.reportValidity()) return;

            setBusy(form, true);
            try {
                await api("/sign-in/email", {
                    method: "POST",
                    body: {
                        email: form.elements.email.value.trim().toLowerCase(),
                        password: form.elements.password.value,
                    },
                });
                window.location.assign("/account/security");
            } catch (error) {
                form.elements.password.value = "";
                if (error instanceof AuthRequestError && error.status === 401) {
                    showStatus(status, "Email or password is incorrect. Check both fields and try again.");
                } else if (error instanceof AuthRequestError && error.status === 429) {
                    showStatus(status, "Too many sign-in attempts. Wait a moment, then try again.");
                } else {
                    showStatus(status, "Sign in is temporarily unavailable. Check your connection and try again.");
                }
            } finally {
                setBusy(form, false);
            }
        });
    }

    function bindSignUp() {
        const form = document.querySelector("#sign-up-form");
        if (!form) return;
        const status = document.querySelector("#sign-up-status");
        const password = form.elements.password;
        const confirmation = form.elements["confirm-password"];

        const validate = () => {
            validatePasswordLength(password);
            validateMatchingPasswords(password, confirmation, "Passwords do not match.");
        };
        password.addEventListener("input", validate);
        confirmation.addEventListener("input", validate);

        form.addEventListener("submit", async (event) => {
            event.preventDefault();
            clearStatus(status);
            validate();
            if (!form.reportValidity()) return;

            setBusy(form, true);
            try {
                await api("/sign-up/email", {
                    method: "POST",
                    body: {
                        name: form.elements.name.value.trim(),
                        email: form.elements.email.value.trim().toLowerCase(),
                        password: password.value,
                    },
                });
                window.location.assign("/account/security");
            } catch (_error) {
                password.value = "";
                confirmation.value = "";
                showStatus(
                    status,
                    "We could not create that account. Check the details or sign in if the email is already registered.",
                );
            } finally {
                setBusy(form, false);
            }
        });
    }

    function bindSessionGate() {
        const gate = document.querySelector("[data-session-gate]");
        if (!gate) return;
        const status = document.querySelector("#session-gate-status");
        const retry = document.querySelector("#session-gate-retry");

        api(gate.dataset.sessionEndpoint)
            .then(() => window.location.replace("/account/security"))
            .catch((error) => {
                if (error instanceof AuthRequestError && error.status === 401) {
                    window.location.replace("/sign-in");
                    return;
                }
                status.textContent = "We could not check your session. Check your connection and try again.";
                retry.hidden = false;
            });
    }

    function formatDate(value) {
        const date = new Date(value);
        if (Number.isNaN(date.getTime())) return "Date unavailable";
        return new Intl.DateTimeFormat(undefined, {
            dateStyle: "medium",
            timeStyle: "short",
        }).format(date);
    }

    function renderSessions(sessions, currentSessionId) {
        const list = document.querySelector("#sessions-list");
        if (!list) return;
        list.replaceChildren();

        if (!Array.isArray(sessions) || sessions.length === 0) {
            const empty = document.createElement("li");
            empty.className = "session-item";
            empty.textContent = "No active sessions were returned.";
            list.append(empty);
            return;
        }

        const ordered = [...sessions].sort((a, b) => {
            const left = new Date(a.createdAt).getTime() || 0;
            const right = new Date(b.createdAt).getTime() || 0;
            return right - left;
        });

        for (const session of ordered) {
            const item = document.createElement("li");
            item.className = "session-item";

            const title = document.createElement("strong");
            title.textContent = `Started ${formatDate(session.createdAt)}`;
            item.append(title);

            if (currentSessionId && session.id === currentSessionId) {
                const badge = document.createElement("span");
                badge.className = "session-badge";
                badge.textContent = "This session";
                item.append(badge);
            }

            const expiry = document.createElement("p");
            expiry.textContent = `Expires ${formatDate(session.expiresAt)}`;
            item.append(expiry);
            list.append(item);
        }
    }

    async function loadSessions() {
        const [current, response] = await Promise.all([
            api("/get-session"),
            api("/list-sessions"),
        ]);
        const sessions = Array.isArray(response)
            ? response.map(({ id, createdAt, expiresAt }) => ({ id, createdAt, expiresAt }))
            : [];
        renderSessions(sessions, current?.session?.id || null);
        return current;
    }

    function bindSignOut() {
        const signOut = document.querySelector("#sign-out");
        if (!signOut) return;
        const status = document.querySelector("#sign-out-status");

        signOut.addEventListener("click", async () => {
            clearStatus(status);
            signOut.disabled = true;
            signOut.textContent = "Signing out…";
            try {
                await api("/sign-out", { method: "POST", body: {} });
                window.location.replace("/sign-in");
            } catch (error) {
                if (error instanceof AuthRequestError && error.status === 401) {
                    window.location.replace("/sign-in");
                    return;
                }
                signOut.disabled = false;
                signOut.textContent = "Sign out";
                showStatus(status, "You could not be signed out. Try again.");
            }
        });
    }

    async function bindAccountPage() {
        const page = document.querySelector("[data-account-page]");
        if (!page) return;

        const loading = document.querySelector("#account-loading");
        const content = document.querySelector("#account-content");
        const sessionStatus = document.querySelector("#sessions-status");
        try {
            const current = await loadSessions();
            document.querySelector("#account-name").textContent = current?.user?.name || "Account owner";
            document.querySelector("#account-email").textContent = current?.user?.email || "";
            document.querySelector("#account-username").value = current?.user?.email || "";
            loading.hidden = true;
            content.hidden = false;
        } catch (error) {
            if (error instanceof AuthRequestError && error.status === 401) {
                window.location.replace("/sign-in");
                return;
            }
            loading.innerHTML = "<p>We could not load your account. Refresh the page to try again.</p>";
            return;
        }

        const passwordForm = document.querySelector("#change-password-form");
        const passwordStatus = document.querySelector("#change-password-status");
        const newPassword = passwordForm.elements["new-password"];
        const confirmation = passwordForm.elements["confirm-new-password"];
        const validate = () => {
            validatePasswordLength(newPassword);
            validateMatchingPasswords(newPassword, confirmation, "New passwords do not match.");
        };
        newPassword.addEventListener("input", validate);
        confirmation.addEventListener("input", validate);

        passwordForm.addEventListener("submit", async (event) => {
            event.preventDefault();
            clearStatus(passwordStatus);
            validate();
            if (!passwordForm.reportValidity()) return;

            setBusy(passwordForm, true);
            try {
                await api("/change-password", {
                    method: "POST",
                    body: {
                        currentPassword: passwordForm.elements["current-password"].value,
                        newPassword: newPassword.value,
                        revokeOtherSessions: passwordForm.elements["revoke-on-change"].checked,
                    },
                });
                passwordForm.reset();
                showStatus(passwordStatus, "Your password has been updated.", "success", false);
            } catch (error) {
                if (error instanceof AuthRequestError && error.status === 401 && !isInvalidCredentials(error)) {
                    window.location.replace("/sign-in");
                    return;
                }
                passwordForm.elements["current-password"].value = "";
                newPassword.value = "";
                confirmation.value = "";
                if (isInvalidCredentials(error)) {
                    showStatus(passwordStatus, "Your current password is incorrect. Try again.");
                } else {
                    showStatus(passwordStatus, "The password service is temporarily unavailable. Try again.");
                }
                return;
            } finally {
                setBusy(passwordForm, false);
            }

            try {
                await loadSessions();
            } catch (error) {
                if (error instanceof AuthRequestError && error.status === 401) {
                    window.location.replace("/sign-in");
                    return;
                }
                showStatus(
                    passwordStatus,
                    "Your password was updated, but the session list could not refresh. Reload the page to try again.",
                    "success",
                    false,
                );
            }
        });

        const revokeOthers = document.querySelector("#revoke-other-sessions");
        revokeOthers.addEventListener("click", async () => {
            clearStatus(sessionStatus);
            revokeOthers.disabled = true;
            revokeOthers.textContent = "Signing out other sessions…";
            try {
                await api("/revoke-other-sessions", { method: "POST", body: {} });
            } catch (error) {
                if (error instanceof AuthRequestError && error.status === 401) {
                    window.location.replace("/sign-in");
                    return;
                }
                showStatus(sessionStatus, "Other sessions could not be signed out. Try again.");
                return;
            } finally {
                revokeOthers.disabled = false;
                revokeOthers.textContent = "Sign out other sessions";
            }

            try {
                await loadSessions();
                showStatus(sessionStatus, "Every other session has been signed out.", "success", false);
            } catch (error) {
                if (error instanceof AuthRequestError && error.status === 401) {
                    window.location.replace("/sign-in");
                    return;
                }
                showStatus(
                    sessionStatus,
                    "Other sessions were signed out, but the list could not refresh. Reload the page to try again.",
                    "success",
                    false,
                );
            }
        });
    }

    bindSessionGate();
    bindSignIn();
    bindSignUp();
    bindSignOut();
    void bindAccountPage();
})();
