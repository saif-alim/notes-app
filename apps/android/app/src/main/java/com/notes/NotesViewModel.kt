package com.notes

import androidx.lifecycle.ViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import uniffi.platform_core.Note
import uniffi.platform_core.NotesCore

class NotesViewModel : ViewModel() {

    private val core = NotesCore()

    private val _notes = MutableStateFlow<List<Note>>(emptyList())
    val notes: StateFlow<List<Note>> = _notes.asStateFlow()

    private val _draft = MutableStateFlow("")
    val draft: StateFlow<String> = _draft.asStateFlow()

    init {
        reload()
    }

    fun setDraft(text: String) {
        _draft.value = text
    }

    fun submit() {
        val body = _draft.value.trim()
        if (body.isEmpty()) return
        core.createNote(body)
        _draft.value = ""
        reload()
    }

    private fun reload() {
        _notes.value = core.listNotes()
    }

    override fun onCleared() {
        super.onCleared()
        core.destroy()
    }
}
