const fs = require('fs')
const process = require('process')
const util = require('util')

const writeFile = util.promisify(fs.writeFile)

// Third Party libraries
const actions = require('@actions/core')
const exec = require('@actions/exec')
const github = require('@actions/github')
const fetch = require('node-fetch')

// Configuration
const argInput = actions.getInput('args')
const args = argInput ? argInput.split() : []

const repo = actions.getInput('repo', { required: true })
const bin = actions.getInput('bin') || repo

const owner = actions.getInput('owner', { required: true })
const regex = new RegExp(actions.getInput('regex', { required: true }))
const token = actions.getInput('token', { required: true })
const tarFile = `/tmp/${bin}.tar.gz`

const octokit = new github.GitHub(token)

async function main () {
  try {
    // Retrieve first release that matched `regex` and download a tar archive of
    // the binary.
    const url = (await octokit.repos.getLatestRelease({ owner, repo }))
      .data
      .assets
      .find(asset => asset.name.match(regex))
      .browser_download_url

    await writeFile(tarFile, await (await fetch(url)).buffer())
    await exec.exec('tar', ['-xvzf', tarFile])

    await exec.exec(`/tmp/${bin}`, args)
  } catch (error) {
    console.log(error)
    process.exit(1)
  }
}

main()
