import * as vscode from 'vscode'
import * as path from 'path'
import * as yaml from 'yaml'

export function activate(context: vscode.ExtensionContext) {
	console.log('Extension "conductor-config-helper" is now active!')

	context.subscriptions.push(
		vscode.workspace.onDidOpenTextDocument(async (document) => {
			if (document.languageId === 'yaml' && document.uri.path.includes('/conductor/config.yaml')) {
				await updateWorkspaceSettings(document, context.extensionPath)
			}
		}),
		registerCompletionProvider()
	)
}

function registerCompletionProvider() {
	return vscode.languages.registerCompletionItemProvider(
		{ language: 'yaml', scheme: 'file', pattern: '**/conductor/config.yaml' },
		{
			provideCompletionItems(document: vscode.TextDocument, position: vscode.Position) {
				const text = document.getText()
				const parsedYaml = yaml.parse(text)

				const sourceIds = parsedYaml.sources?.map((source: any) => source.id) ?? []
				const cacheIds = parsedYaml.cache_stores?.map((cache: any) => cache.id) ?? []

				const completionItems: any = []
				const linePrefix = document.lineAt(position).text.substr(0, position.character)

				// Suggest source IDs for 'from' field in endpoints
				if (/from:\s*$/.test(linePrefix)) {
					sourceIds.forEach((id: any) => {
						const completionItem = new vscode.CompletionItem(id, vscode.CompletionItemKind.Value)
						completionItem.detail = 'Source ID'
						completionItems.push(completionItem)
					})
				}

				// Suggest cache IDs for 'cache' field in response_cache
				if (/cache:\s*$/.test(linePrefix)) {
					cacheIds.forEach((id: any) => {
						const completionItem = new vscode.CompletionItem(id, vscode.CompletionItemKind.Value)
						completionItem.detail = 'Cache Store ID'
						completionItems.push(completionItem)
					})
				}

				return completionItems
			}
		}
	)
}

async function updateWorkspaceSettings(document: vscode.TextDocument, extensionPath: string) {
	const schemaPath = vscode.Uri.file(path.join(extensionPath, 'src', 'conductor.schema.json')).toString()
	const yamlSettings = vscode.workspace.getConfiguration('yaml', null)
	const currentSchemas = yamlSettings.get('schemas') || {} as any

	// Set the schema for conductor/config.yaml
	currentSchemas[schemaPath] = [document.uri.toString()]

	await yamlSettings.update('schemas', currentSchemas, vscode.ConfigurationTarget.Workspace)
}

export function deactivate() { }
