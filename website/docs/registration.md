---
title: Registration & Project Setup
sidebar_position: 0
---

# Registration & Project Setup

This guide walks you through the process of registering for an OpenSecret account, creating an organization, and setting up your first project to obtain the client ID needed for your application.

## Overview

Before you can use the OpenSecret SDK in your application, you need to:

1. Register for an OpenSecret account
2. Create or join an organization
3. Create a project within your organization
4. Obtain the client ID for your project

## Registering for an Account

To register for an OpenSecret account:

1. Visit [https://opensecret.cloud](https://opensecret.cloud)
2. Click on the "Sign Up" button
3. Fill in your registration details:
   - Name
   - Email address
   - Password (min 8 characters)
   - Invitation code (if required)
4. Click "Create Account"
5. Verify your email address by clicking the link sent to your email

![Registration Page](https://placeholder-for-signup-screenshot.png)

:::note Invitation Codes
During the preview phase, an invitation code may be required to register. If you need an invitation code, please contact the OpenSecret team at [team@opensecret.cloud](mailto:team@opensecret.cloud).
:::

## Creating an Organization

After registering and verifying your email:

1. Log in to your OpenSecret account
2. If you don't already have an organization, you'll be prompted to create one:
   - Enter your organization name
   - Choose a unique slug (URL identifier)
   - Provide a brief description (optional)
3. Click "Create Organization"

![Create Organization](https://placeholder-for-create-org-screenshot.png)

Organizations help you manage projects and team members. You can create multiple organizations if needed.

## Inviting Team Members

To invite team members to your organization:

1. Navigate to your organization settings
2. Click on the "Members" tab
3. Click "Invite Member"
4. Enter the email address of the person you want to invite
5. Select their role (Admin, Developer, or Viewer)
6. Click "Send Invitation"

![Invite Team Members](https://placeholder-for-invite-members-screenshot.png)

## Creating a Project

Once you have an organization:

1. Go to your organization dashboard
2. Click "Create New Project"
3. Enter a project name
4. (Optional) Provide a description for your project
5. Choose appropriate settings for your project:
   - Environment (Development, Staging, Production)
   - Authentication methods (Email, Guest, Social logins)
6. Click "Create Project"

![Create Project](https://placeholder-for-create-project-screenshot.png)

## Getting Your Client ID

After creating a project, you need to get your client ID:

1. Navigate to your project in the dashboard
2. Go to the "Settings" tab
3. Look for the "Client ID" section
4. Copy the UUID shown - this is your client ID

![Client ID](https://placeholder-for-client-id-screenshot.png)

The client ID is a UUID that identifies your project and is required when setting up the OpenSecretProvider in your application:

```tsx
<OpenSecretProvider
  apiUrl="https://api.opensecret.cloud"
  clientId="your-client-id-here"
>
  <YourApp />
</OpenSecretProvider>
```

## Project Configuration

### Authentication Settings

You can configure authentication methods for your project:

1. Go to your project settings
2. Navigate to the "Authentication" tab
3. Enable or disable authentication methods:
   - Email/Password authentication
   - Guest accounts
   - Social logins (GitHub, Google)
4. Set policies like password requirements and session duration
5. Save your changes

![Authentication Settings](https://placeholder-for-auth-settings-screenshot.png)

### Invite Codes

To control who can access your application, you can manage invite codes:

1. Go to your project settings
2. Navigate to the "Invite Codes" tab
3. Click "Generate New Code"
4. Set an expiration date and usage limit
5. Click "Create Code"
6. Share the generated code with your users

![Invite Codes](https://placeholder-for-invite-codes-screenshot.png)

Users will need to provide this invite code when signing up for your application.

### API Configuration

Configure API settings for your project:

1. Go to your project settings
2. Navigate to the "API" tab
3. Set CORS origins to allow requests from your application domains
4. Configure rate limits for API endpoints
5. Save your changes

![API Configuration](https://placeholder-for-api-config-screenshot.png)

## Environments

OpenSecret supports multiple environments (Development, Staging, Production) for your projects:

1. Go to your organization dashboard
2. Click "Create New Project"
3. Select the appropriate environment for your project
4. Each environment gets its own client ID and configuration

This allows you to have separate settings and data for different stages of your application development.

## Next Steps

Now that you have registered and set up your project, you can:

1. [Get started with the SDK](./guides/getting-started)
2. [Set up authentication in your app](./guides/authentication)
3. [Explore key-value storage](./guides/key-value-storage)
4. [Learn about cryptographic operations](./guides/cryptographic-operations)

:::tip
Keep your client ID secure but not secret. While it's not sensitive like an API key (since all operations require user authentication), it should be handled respectfully as it identifies your application.
:::

## Troubleshooting

If you encounter issues during registration or project setup:

### Common Registration Issues

- **Email verification link expired**: Request a new verification email from the login page
- **Invitation code invalid**: Check if the code has expired or reached its usage limit
- **Organization creation failed**: Ensure the organization slug is unique

### Project Setup Issues

- **Cannot create project**: Verify you have admin permissions in your organization
- **Settings not saving**: Refresh the page and try again
- **Client ID not visible**: Ensure you have the correct permissions to view project settings

For additional help, contact [support@opensecret.cloud](mailto:support@opensecret.cloud) or check the [OpenSecret Community Forum](https://community.opensecret.cloud).