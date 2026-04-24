package com.notes

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import uniffi.platform_core.Note

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun NotesListScreen(viewModel: NotesViewModel) {
    val notes by viewModel.notes.collectAsStateWithLifecycle()
    val draft by viewModel.draft.collectAsStateWithLifecycle()

    Scaffold(
        topBar = { TopAppBar(title = { Text("Notes") }) }
    ) { padding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
                .padding(horizontal = 16.dp)
        ) {
            LazyColumn(
                modifier = Modifier.weight(1f),
                verticalArrangement = Arrangement.spacedBy(8.dp),
                contentPadding = PaddingValues(vertical = 8.dp)
            ) {
                items(notes, key = { it.id }) { note ->
                    NoteRow(note)
                }
            }

            HorizontalDivider()
            Spacer(Modifier.height(8.dp))

            OutlinedTextField(
                value = draft,
                onValueChange = viewModel::setDraft,
                modifier = Modifier.fillMaxWidth(),
                placeholder = { Text("New note…") },
                singleLine = true,
                trailingIcon = {
                    TextButton(
                        onClick = viewModel::submit,
                        enabled = draft.isNotBlank()
                    ) {
                        Text("Add")
                    }
                }
            )

            Spacer(Modifier.height(8.dp))
        }
    }
}

@Composable
private fun NoteRow(note: Note) {
    Card(modifier = Modifier.fillMaxWidth()) {
        Column(modifier = Modifier.padding(12.dp)) {
            Text(
                text = note.body,
                style = MaterialTheme.typography.bodyMedium,
                maxLines = 3
            )
        }
    }
}
