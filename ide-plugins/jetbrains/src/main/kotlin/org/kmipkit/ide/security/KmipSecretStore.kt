package org.kmipkit.ide.security

import com.intellij.credentialStore.CredentialAttributes
import com.intellij.credentialStore.Credentials
import com.intellij.credentialStore.generateServiceName
import com.intellij.ide.passwordSafe.PasswordSafe

object KmipSecretStore {
    private fun attributesFor(profileName: String, key: String): CredentialAttributes {
        return CredentialAttributes(generateServiceName("KMIPKit", "$profileName:$key"))
    }

    fun setSecret(profileName: String, key: String, secret: String) {
        PasswordSafe.instance.set(attributesFor(profileName, key), Credentials(key, secret))
    }

    fun getSecret(profileName: String, key: String): String? {
        return PasswordSafe.instance.getPassword(attributesFor(profileName, key))
    }

    fun deleteSecret(profileName: String, key: String) {
        PasswordSafe.instance.set(attributesFor(profileName, key), null)
    }
}
